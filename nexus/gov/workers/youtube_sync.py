"""
NEXUS GOV -- YouTube Sync Worker.

Downloads videos from politician channels and parliamentary TV.
Uses yt-dlp for download, emits GOV_VIDEO_DOWNLOADED for transcription.
"""
from __future__ import annotations

import asyncio
import json
from pathlib import Path
from typing import Any

from loguru import logger

from nexus.engine import ReactiveWorker, NexusEvent
from nexus.gov.events import GovEventType
from nexus.config import settings

import httpx

SEARXNG_URL = "http://localhost:8888/search"
VIDEO_DIR = Path(settings.data_dir) / "gov_videos"

# Parliamentary TV channels -- scanned every tick for new content
PARLIAMENTARY_CHANNELS = [
    {"url": "https://www.youtube.com/@LCP", "name": "LCP"},
    {"url": "https://www.youtube.com/@publicsenat", "name": "Public Sénat"},
    {"url": "https://www.youtube.com/@AssembleeNationale", "name": "Assemblée Nationale"},
]

# Max recent videos to fetch per parliamentary channel
_CHANNEL_PLAYLIST_LIMIT = 15


class GovYouTubeSyncWorker(ReactiveWorker):
    name = "gov_youtube_sync"
    subscriptions = [GovEventType.TICK_DAILY]

    def __init__(self, bus: Any, db: Any) -> None:
        super().__init__(bus)
        self._db = db

    async def handle(self, event: NexusEvent) -> list[NexusEvent]:
        VIDEO_DIR.mkdir(parents=True, exist_ok=True)
        output: list[NexusEvent] = []

        # --- Part 1: Search for politician-specific videos via SearXNG ---
        politician_events = await self._sync_politician_videos(event)
        output.extend(politician_events)

        # --- Part 2: Scan parliamentary channels for new videos ---
        channel_events = await self._sync_parliamentary_channels(event)
        output.extend(channel_events)

        if output:
            logger.info("YouTube sync: {} videos downloaded", len(output))
        return output

    # ------------------------------------------------------------------
    # Part 1: Politician-specific search (existing logic, improved)
    # ------------------------------------------------------------------

    async def _sync_politician_videos(self, event: NexusEvent) -> list[NexusEvent]:
        """Search SearXNG for recent YouTube videos mentioning politicians."""
        output: list[NexusEvent] = []
        politicians = await self._db.list_politicians(limit=50)

        for pol in politicians[:20]:  # Limit per tick
            try:
                async with httpx.AsyncClient(timeout=15.0) as client:
                    resp = await client.get(SEARXNG_URL, params={
                        "q": f'"{pol["name"]}" interview OR discours OR assemblee',
                        "format": "json",
                        "categories": "videos",
                        "language": "fr",
                    })
                    if resp.status_code != 200:
                        continue
                    data = resp.json()

                for r in data.get("results", [])[:2]:
                    url = r.get("url", "")
                    if "youtube.com" not in url and "youtu.be" not in url:
                        continue

                    title = r.get("title", "")

                    # Check if already processed
                    existing = await self._db.list_transcriptions_by_politician(pol["id"], limit=1000)
                    existing_urls = {t.get("source_url", "") for t in existing}
                    if url in existing_urls:
                        continue

                    # Download audio only with yt-dlp
                    audio_path = await self._download_audio(url, f"{pol['id']}_{hash(url) & 0xFFFFFFFF}")
                    if audio_path is None:
                        continue

                    output.append(NexusEvent(
                        event_type=GovEventType.GOV_VIDEO_DOWNLOADED,
                        case_id="gov",
                        payload={
                            "politician_id": pol["id"],
                            "politician_name": pol["name"],
                            "video_url": url,
                            "audio_path": str(audio_path),
                            "title": title,
                        },
                        source_worker=self.name,
                        parent_event_id=event.event_id,
                    ))
                    logger.info("YouTube downloaded: '{}' for {}", title[:50], pol["name"])

                await asyncio.sleep(3.0)
            except Exception as exc:
                logger.debug("YouTube sync skip '{}': {}", pol["name"], exc)

        return output

    # ------------------------------------------------------------------
    # Part 2: Parliamentary channel scanning
    # ------------------------------------------------------------------

    async def _sync_parliamentary_channels(self, event: NexusEvent) -> list[NexusEvent]:
        """Scan PARLIAMENTARY_CHANNELS for new videos using yt-dlp --flat-playlist."""
        output: list[NexusEvent] = []

        for channel in PARLIAMENTARY_CHANNELS:
            try:
                videos = await self._list_channel_videos(
                    channel["url"], limit=_CHANNEL_PLAYLIST_LIMIT
                )
                if not videos:
                    logger.debug("No videos listed for channel {}", channel["name"])
                    continue

                # Check which videos we already have (use politician_id=None for channel videos)
                # Parliamentary channel videos are not tied to a specific politician
                for vid in videos:
                    video_url = vid.get("url", vid.get("webpage_url", ""))
                    if not video_url:
                        continue

                    # Normalize URL -- yt-dlp flat-playlist may return relative IDs
                    if not video_url.startswith("http"):
                        video_url = f"https://www.youtube.com/watch?v={video_url}"

                    title = vid.get("title", channel["name"])

                    # Check if already downloaded by looking for existing audio file
                    file_key = f"channel_{hash(video_url) & 0xFFFFFFFF}"
                    audio_path = VIDEO_DIR / f"{file_key}.mp3"
                    if audio_path.exists():
                        continue

                    # Download audio
                    downloaded = await self._download_audio(video_url, file_key)
                    if downloaded is None:
                        continue

                    output.append(NexusEvent(
                        event_type=GovEventType.GOV_VIDEO_DOWNLOADED,
                        case_id="gov",
                        payload={
                            "politician_id": None,
                            "politician_name": None,
                            "video_url": video_url,
                            "audio_path": str(downloaded),
                            "title": title,
                            "channel": channel["name"],
                            "channel_url": channel["url"],
                        },
                        source_worker=self.name,
                        parent_event_id=event.event_id,
                    ))
                    logger.info(
                        "YouTube channel '{}': downloaded '{}'",
                        channel["name"], title[:50],
                    )

                    # Rate-limit between downloads
                    await asyncio.sleep(3.0)

            except Exception as exc:
                logger.debug(
                    "YouTube channel scan failed for '{}': {}",
                    channel["name"], exc,
                )

        return output

    async def _list_channel_videos(
        self, channel_url: str, limit: int = 15
    ) -> list[dict]:
        """Use yt-dlp --flat-playlist to list recent videos from a channel.

        Returns a list of dicts with at least 'url' and 'title' keys.
        """
        try:
            proc = await asyncio.create_subprocess_exec(
                "yt-dlp",
                "--flat-playlist",
                "--playlist-end", str(limit),
                "--dump-json",
                f"{channel_url}/videos",
                stdout=asyncio.subprocess.PIPE,
                stderr=asyncio.subprocess.DEVNULL,
            )
            stdout, _ = await asyncio.wait_for(proc.communicate(), timeout=60)
            if proc.returncode != 0:
                return []

            # yt-dlp --dump-json outputs one JSON object per line
            videos = []
            for line in stdout.decode("utf-8", errors="replace").strip().splitlines():
                line = line.strip()
                if not line:
                    continue
                try:
                    videos.append(json.loads(line))
                except json.JSONDecodeError:
                    continue

            return videos

        except (asyncio.TimeoutError, FileNotFoundError) as exc:
            logger.debug("yt-dlp flat-playlist failed for {}: {}", channel_url, exc)
            return []

    # ------------------------------------------------------------------
    # Shared download helper
    # ------------------------------------------------------------------

    async def _download_audio(self, url: str, file_key: str) -> Path | None:
        """Download audio from a YouTube URL using yt-dlp.

        Uses -o with .%(ext)s so yt-dlp can properly handle the extension,
        then renames to .mp3. Returns the audio path or None on failure.
        """
        # yt-dlp needs the output template without extension (it adds .mp3 via --audio-format)
        output_template = str(VIDEO_DIR / f"{file_key}.%(ext)s")
        audio_path = VIDEO_DIR / f"{file_key}.mp3"

        if audio_path.exists():
            return audio_path

        try:
            proc = await asyncio.create_subprocess_exec(
                "yt-dlp",
                "-x",
                "--audio-format", "mp3",
                "--audio-quality", "5",
                "--max-filesize", "50m",
                "-o", output_template,
                url,
                stdout=asyncio.subprocess.DEVNULL,
                stderr=asyncio.subprocess.DEVNULL,
            )
            await asyncio.wait_for(proc.wait(), timeout=120)
            if proc.returncode != 0:
                return None
        except (asyncio.TimeoutError, FileNotFoundError):
            logger.debug("yt-dlp download failed for {}", url[:60])
            return None

        # yt-dlp with -x --audio-format mp3 creates <file_key>.mp3
        if audio_path.exists():
            return audio_path

        # Fallback: sometimes the file keeps .webm or .m4a extension
        # Check for any file matching the key pattern
        for candidate in VIDEO_DIR.glob(f"{file_key}.*"):
            if candidate.suffix in (".mp3", ".m4a", ".opus", ".webm", ".ogg"):
                return candidate

        return None
