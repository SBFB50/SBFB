"""
NEXUS GOV -- Vision Worker.

Extracts visual intelligence from downloaded videos:
- Key frame extraction (scene changes + interval)
- OCR on frames (TV chyrons, banners, on-screen text)
- Person detection + CLIP matching against politician photos
- Scene classification (parliament, TV studio, press conference, etc.)
"""
from __future__ import annotations

import asyncio
import os
import tempfile
from pathlib import Path
from typing import Any

from loguru import logger

from nexus.engine import ReactiveWorker, NexusEvent
from nexus.gov.events import GovEventType

# ---------------------------------------------------------------------------
# Scene labels for CLIP zero-shot classification
# ---------------------------------------------------------------------------

SCENE_LABELS = [
    "a parliament session in a hemicycle",
    "a TV news studio",
    "a press conference",
    "a political rally with crowd",
    "a committee hearing",
    "an interview setting",
    "a street protest",
    "an outdoor political event",
    "a formal government building interior",
]

# ---------------------------------------------------------------------------
# Video file extensions to look for alongside audio
# ---------------------------------------------------------------------------

_VIDEO_EXTENSIONS = (".mp4", ".webm", ".mkv", ".avi", ".mov")


class GovVisionWorker(ReactiveWorker):
    """Extracts visual intelligence from downloaded videos.

    Subscribes to GOV_VIDEO_DOWNLOADED (same event as GovTranscriptionWorker)
    and runs in parallel: the transcription worker handles audio, this one
    handles video frames.

    Pipeline per video:
    1. Locate the video file (alongside the audio .mp3 file)
    2. Extract key frames at scene changes + every 60s fallback
    3. OCR each frame (full frame + chyron zone)
    4. Classify scenes via CLIP zero-shot
    5. Store OCR chyrons as alerts, emit GOV_IMAGE_ADDED downstream
    """

    name = "gov_vision"
    subscriptions = [GovEventType.GOV_VIDEO_DOWNLOADED]

    def __init__(self, bus: Any, db: Any) -> None:
        super().__init__(bus)
        self._db = db
        self._ocr = None  # Lazy-loaded PaddleOCR; False = sentinel for unavailable
        self._embedder = None  # Lazy-loaded VisualEmbedder (CLIP)

    # ------------------------------------------------------------------
    # Video file discovery
    # ------------------------------------------------------------------

    @staticmethod
    def _find_video(audio_path: str) -> str | None:
        """Find a video file alongside the audio file (.mp3 -> .mp4/.webm/etc).

        Returns the first matching video path, or None.
        """
        if not audio_path:
            return None

        base = Path(audio_path)

        # Check same directory, same stem, different extension
        for ext in _VIDEO_EXTENSIONS:
            candidate = base.with_suffix(ext)
            if candidate.exists():
                return str(candidate)

        # Also check if audio_path itself is a video file
        if base.exists() and base.suffix.lower() in _VIDEO_EXTENSIONS:
            return str(base)

        return None

    # ------------------------------------------------------------------
    # Frame extraction (OpenCV + scenedetect)
    # ------------------------------------------------------------------

    @staticmethod
    def _extract_key_frames(
        video_path: str, max_frames: int = 30
    ) -> list[tuple[float, Any]]:
        """Extract frames at scene changes + every 60s fallback.

        Returns list of (timestamp_seconds, numpy_ndarray) tuples.
        """
        try:
            import cv2
        except ImportError:
            logger.warning("opencv-python not installed, frame extraction disabled")
            return []

        # Detect scene changes (optional dependency)
        scenes: list = []
        try:
            from scenedetect import detect, AdaptiveDetector

            scenes = detect(video_path, AdaptiveDetector(adaptive_threshold=3.0))
        except ImportError:
            logger.debug("scenedetect not installed, using interval-only extraction")
        except Exception as exc:
            logger.debug("scenedetect failed ({}), falling back to intervals", exc)

        cap = cv2.VideoCapture(video_path)
        if not cap.isOpened():
            logger.warning("Cannot open video: {}", video_path)
            return []

        fps = cap.get(cv2.CAP_PROP_FPS) or 25.0
        total_frames = int(cap.get(cv2.CAP_PROP_FRAME_COUNT))

        if total_frames <= 0:
            cap.release()
            return []

        # Build target frame numbers: scene midpoints + every 60s
        targets: set[int] = set()

        for start, end in scenes:
            mid = (start.get_frames() + end.get_frames()) // 2
            targets.add(mid)

        interval_frames = int(fps * 60)  # 1 frame per 60s
        if interval_frames > 0:
            for f in range(0, total_frames, interval_frames):
                targets.add(f)

        # Always include first frame
        targets.add(0)

        # Read frames
        frames: list[tuple[float, Any]] = []
        for frame_no in sorted(targets)[:max_frames]:
            cap.set(cv2.CAP_PROP_POS_FRAMES, frame_no)
            ok, frame = cap.read()
            if ok:
                timestamp = frame_no / fps
                frames.append((timestamp, frame))

        cap.release()
        return frames

    # ------------------------------------------------------------------
    # OCR (PaddleOCR, lazy-loaded)
    # ------------------------------------------------------------------

    def _get_ocr(self) -> Any | None:
        """Lazy-load PaddleOCR. Returns the instance or None if unavailable."""
        if self._ocr is None:
            try:
                from paddleocr import PaddleOCR

                self._ocr = PaddleOCR(
                    use_angle_cls=True,
                    lang="fr",
                    use_gpu=True,
                    show_log=False,
                )
                logger.info("PaddleOCR loaded (fr, GPU)")
            except ImportError:
                logger.warning("paddleocr not installed, OCR disabled")
                self._ocr = False  # sentinel: not available
            except Exception as exc:
                logger.warning("PaddleOCR init failed ({}), OCR disabled", exc)
                self._ocr = False

        return self._ocr if self._ocr is not False else None

    @staticmethod
    def _run_ocr(ocr: Any, img: Any) -> list[dict[str, Any]]:
        """Run PaddleOCR on a single image (numpy array).

        Returns list of {text, confidence, is_chyron?} dicts.
        """
        try:
            result = ocr.ocr(img, cls=True)
        except Exception as exc:
            logger.debug("OCR failed on frame: {}", exc)
            return []

        texts: list[dict[str, Any]] = []
        if not result or not result[0]:
            return texts

        for line in result[0]:
            bbox, (text, conf) = line[0], line[1]
            if conf > 0.6 and len(text.strip()) > 2:
                texts.append({
                    "text": text.strip(),
                    "confidence": round(conf, 2),
                })

        return texts

    def _extract_text(self, frame: Any) -> list[dict[str, Any]]:
        """Extract text from frame, focusing on bottom 30% (chyron zone).

        Returns list of {text, confidence, is_chyron} dicts.
        """
        ocr = self._get_ocr()
        if not ocr:
            return []

        h = frame.shape[0]

        # Full frame OCR
        full_texts = self._run_ocr(ocr, frame)

        # Chyron-specific OCR (bottom 30% of the frame)
        chyron_region = frame[int(h * 0.7):, :]
        chyron_texts = self._run_ocr(ocr, chyron_region)
        for t in chyron_texts:
            t["is_chyron"] = True

        # Combine (deduplicate by text content)
        seen: set[str] = set()
        combined: list[dict[str, Any]] = []

        # Chyron texts first (higher priority)
        for t in chyron_texts:
            if t["text"] not in seen:
                seen.add(t["text"])
                combined.append(t)

        for t in full_texts:
            if t["text"] not in seen:
                seen.add(t["text"])
                combined.append(t)

        return combined

    # ------------------------------------------------------------------
    # Scene classification (CLIP zero-shot)
    # ------------------------------------------------------------------

    def _get_embedder(self) -> Any | None:
        """Lazy-load the VisualEmbedder for CLIP scene classification."""
        if self._embedder is None:
            try:
                from nexus.vision.embeddings import VisualEmbedder

                self._embedder = VisualEmbedder()
                logger.info("VisualEmbedder loaded for scene classification")
            except ImportError:
                logger.warning(
                    "nexus.vision.embeddings not available, scene classification disabled"
                )
                self._embedder = False  # sentinel
            except Exception as exc:
                logger.warning("VisualEmbedder init failed ({})", exc)
                self._embedder = False

        return self._embedder if self._embedder is not False else None

    async def _classify_scene(self, frame_path: str) -> tuple[str, float]:
        """Zero-shot scene classification via CLIP.

        Returns (best_label, confidence_score).
        """
        embedder = self._get_embedder()
        if not embedder:
            return "unknown", 0.0

        try:
            img_emb = await asyncio.to_thread(embedder.embed_image_clip, frame_path)

            best_label = "unknown"
            best_score = 0.0

            for label in SCENE_LABELS:
                text_emb = await asyncio.to_thread(embedder.embed_text_clip, label)
                # Cosine similarity (embeddings are already L2-normalized)
                score = sum(a * b for a, b in zip(img_emb, text_emb))
                if score > best_score:
                    best_score = score
                    best_label = label

            return best_label, round(best_score, 3)

        except Exception as exc:
            logger.debug("Scene classification failed: {}", exc)
            return "unknown", 0.0

    # ------------------------------------------------------------------
    # Main handler
    # ------------------------------------------------------------------

    async def handle(self, event: NexusEvent) -> list[NexusEvent]:
        """Process a GOV_VIDEO_DOWNLOADED event: extract frames, OCR, classify."""
        payload = event.payload
        video_url = payload.get("video_url", "")
        audio_path = payload.get("audio_path", "")
        politician_id = payload.get("politician_id")
        title = payload.get("title", "")

        # Find video file (might be alongside audio)
        video_path = self._find_video(audio_path)
        if not video_path:
            logger.debug("No video file for {}, skipping vision", video_url)
            return []

        # Extract key frames (CPU-intensive: run in thread)
        frames = await asyncio.to_thread(
            self._extract_key_frames, video_path
        )
        if not frames:
            logger.debug("No frames extracted from {}", video_path)
            return []

        logger.info(
            "Vision: {} frames from '{}' ({})",
            len(frames), title[:50], video_url,
        )

        all_ocr: list[dict[str, Any]] = []
        scene_types: list[dict[str, Any]] = []

        for idx, (ts, frame) in enumerate(frames):
            # --- OCR ---
            texts = await asyncio.to_thread(self._extract_text, frame)
            if texts:
                all_ocr.append({"timestamp": round(ts, 2), "texts": texts})

            # --- Scene classification (first frame + every 5th) ---
            if idx == 0 or idx % 5 == 0:
                tmp_path = os.path.join(
                    tempfile.gettempdir(),
                    f"nexus_gov_frame_{ts:.0f}.jpg",
                )
                try:
                    import cv2

                    cv2.imwrite(tmp_path, frame)
                    scene_label, scene_conf = await self._classify_scene(tmp_path)
                    scene_types.append({
                        "timestamp": round(ts, 2),
                        "scene": scene_label,
                        "confidence": scene_conf,
                    })
                except ImportError:
                    pass  # cv2 not available
                except Exception as exc:
                    logger.debug("Scene classification error at {:.0f}s: {}", ts, exc)
                finally:
                    # Clean up temp frame file
                    try:
                        os.unlink(tmp_path)
                    except OSError:
                        pass

        # --- Collect chyron texts ---
        chyron_texts = [
            t["text"]
            for entry in all_ocr
            for t in entry["texts"]
            if t.get("is_chyron")
        ]

        # --- Create alert for significant chyron findings ---
        if chyron_texts:
            try:
                await self._db.create_alert(
                    alert_type="vision_chyron",
                    title=f"Chyrons detectes: {title[:60]}",
                    description=f"Textes: {'; '.join(chyron_texts[:10])}",
                    severity="low",
                    politician_id=politician_id,
                )
            except Exception as exc:
                logger.warning("Failed to create chyron alert: {}", exc)

        # --- Log summary ---
        logger.info(
            "Vision complete for '{}': {} OCR entries, {} scenes, {} chyrons",
            title[:40],
            len(all_ocr),
            len(scene_types),
            len(chyron_texts),
        )

        # --- Emit GOV_IMAGE_ADDED for downstream processing ---
        return [NexusEvent(
            event_type=GovEventType.GOV_IMAGE_ADDED,
            case_id="gov",
            payload={
                "video_url": video_url,
                "politician_id": politician_id,
                "title": title,
                "frames_analyzed": len(frames),
                "ocr_results": len(all_ocr),
                "ocr_texts": [
                    t["text"]
                    for entry in all_ocr
                    for t in entry["texts"]
                ][:50],
                "scenes": scene_types[:5],
                "chyrons": chyron_texts[:20],
            },
            source_worker=self.name,
            parent_event_id=event.event_id,
        )]
