"""
NEXUS GOV -- YouTube Sync Worker.

Downloads videos from politician channels and parliamentary TV.
Uses yt-dlp for download, emits GOV_VIDEO_DOWNLOADED for transcription.
"""
from __future__ import annotations

import asyncio
from pathlib import Path
from typing import Any

from loguru import logger

from nexus.engine import ReactiveWorker, NexusEvent
from nexus.gov.events import GovEventType
from nexus.config import settings

import httpx

SEARXNG_URL = "http://localhost:8888/search"
VIDEO_DIR = Path(settings.data_dir) / "gov_videos"

# Parliamentary TV channels
PARLIAMENTARY_CHANNELS = [
    "https://www.youtube.com/@LCP",
    "https://www.youtube.com/@publicsenat",
    "https://www.youtube.com/@AssembleeNationale",
]


class GovYouTubeSyncWorker(ReactiveWorker):
    name = "gov_youtube_sync"
    subscriptions = [GovEventType.TICK_DAILY]

    def __init__(self, bus: Any, db: Any) -> None:
        super().__init__(bus)
        self._db = db

    async def handle(self, event: NexusEvent) -> list[NexusEvent]:
        VIDEO_DIR.mkdir(parents=True, exist_ok=True)
        output: list[NexusEvent] = []

        # Strategy: search SearXNG for recent YouTube videos mentioning politicians
        politicians = await self._db.list_politicians(limit=50)  # Top 50 by recent activity

        for pol in politicians[:20]:  # Limit per tick
            try:
                # Search for recent YouTube videos
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
                    audio_path = VIDEO_DIR / f"{pol['id']}_{hash(url) & 0xFFFFFFFF}.mp3"
                    if not audio_path.exists():
                        try:
                            proc = await asyncio.create_subprocess_exec(
                                "yt-dlp", "-x", "--audio-format", "mp3",
                                "--audio-quality", "5",
                                "--max-filesize", "50m",
                                "-o", str(audio_path),
                                url,
                                stdout=asyncio.subprocess.DEVNULL,
                                stderr=asyncio.subprocess.DEVNULL,
                            )
                            await asyncio.wait_for(proc.wait(), timeout=120)
                            if proc.returncode != 0:
                                continue
                        except (asyncio.TimeoutError, FileNotFoundError):
                            logger.debug("yt-dlp failed for {}", url[:60])
                            continue

                    # Emit event for transcription worker
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

        if output:
            logger.info("YouTube sync: {} videos downloaded", len(output))
        return output
