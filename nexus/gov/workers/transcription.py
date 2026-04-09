"""
NEXUS GOV -- Transcription Worker.

Transcribes downloaded audio/video using faster-whisper.
Produces timestamped transcriptions stored in gov_transcriptions.
"""
from __future__ import annotations

import json
from pathlib import Path
from typing import Any

from loguru import logger

from nexus.events.worker import ReactiveWorker
from nexus.events.types import NexusEvent
from nexus.gov.events import GovEventType


class GovTranscriptionWorker(ReactiveWorker):
    name = "gov_transcription"
    subscriptions = [GovEventType.GOV_VIDEO_DOWNLOADED]

    def __init__(self, bus: Any, db: Any) -> None:
        super().__init__(bus)
        self._db = db
        self._model = None

    def _get_model(self):
        """Lazy-load faster-whisper model."""
        if self._model is None:
            try:
                from faster_whisper import WhisperModel
                self._model = WhisperModel("large-v3", device="cuda", compute_type="float16")
                logger.info("faster-whisper model loaded (large-v3, CUDA)")
            except Exception:
                try:
                    from faster_whisper import WhisperModel
                    self._model = WhisperModel("base", device="cpu", compute_type="int8")
                    logger.info("faster-whisper model loaded (base, CPU fallback)")
                except ImportError:
                    logger.warning("faster-whisper not installed, transcription disabled")
                    return None
        return self._model

    async def handle(self, event: NexusEvent) -> list[NexusEvent]:
        model = self._get_model()
        if model is None:
            return []

        audio_path = event.payload.get("audio_path", "")
        if not audio_path or not Path(audio_path).exists():
            logger.warning("Audio file not found: {}", audio_path)
            return []

        politician_id = event.payload.get("politician_id")
        video_url = event.payload.get("video_url", "")
        title = event.payload.get("title", "Transcription")

        try:
            # Run transcription in thread (CPU/GPU intensive)
            import asyncio
            segments, info = await asyncio.to_thread(
                model.transcribe, audio_path, language="fr", beam_size=5
            )

            # Build timestamped text
            full_text = []
            timestamped = []
            for segment in segments:
                full_text.append(segment.text.strip())
                timestamped.append({
                    "start": round(segment.start, 2),
                    "end": round(segment.end, 2),
                    "text": segment.text.strip(),
                })

            transcription_text = " ".join(full_text)
            duration = int(info.duration) if info.duration else 0

            if not transcription_text:
                return []

            # Store in DB
            record = await self._db.create_transcription(
                source_type="youtube",
                source_url=video_url,
                politician_id=politician_id,
                title=title,
                transcription=transcription_text,
                timestamped_text=json.dumps(timestamped, ensure_ascii=False),
                duration_seconds=duration,
                model_used="faster-whisper-large-v3",
            )

            logger.info(
                "Transcribed '{}': {} chars, {}s",
                title[:40], len(transcription_text), duration,
            )

            # Clean up audio file
            try:
                Path(audio_path).unlink(missing_ok=True)
            except Exception:
                pass

            return [NexusEvent(
                event_type=GovEventType.GOV_TRANSCRIPTION_READY,
                case_id="gov",
                payload={
                    "transcription_id": record["id"],
                    "politician_id": politician_id,
                    "text_length": len(transcription_text),
                    "duration": duration,
                    "title": title,
                },
                source_worker=self.name,
                parent_event_id=event.event_id,
            )]

        except Exception as exc:
            logger.error("Transcription failed for '{}': {}", title[:40], exc)
            return []
