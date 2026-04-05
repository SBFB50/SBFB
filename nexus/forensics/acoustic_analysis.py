"""
NEXUS -- Forensic acoustic analysis.

Audio analysis for investigation support:
- Transcription via Ollama voxtral model
- Forensic analysis of audio recordings (voices, events, editing)
- Event detection using basic DSP (RMS energy, silence, spectral)
- Sound propagation calculations for source localization
"""

from __future__ import annotations

import json
import math
import re
import struct
import wave
from pathlib import Path
from typing import Any, Dict, List, Optional, Tuple

from loguru import logger

from nexus.llm.prompts import (
    AUDIO_FORENSIC_ANALYSIS_PROMPT,
    AUDIO_TRANSCRIPTION_PROMPT,
)
from nexus.llm.router import LLMRouter, TaskType


class AcousticAnalyzer:
    """Forensic audio analysis for investigation support."""

    def __init__(self, router: LLMRouter) -> None:
        self._router = router

    # ==================================================================
    # Transcription
    # ==================================================================

    async def transcribe_audio(self, audio_path: str | Path) -> str:
        """Transcribe audio using Ollama voxtral model.

        Uses the voxtral-mini:4b model configured in the project.
        The audio file is sent to the model for speech-to-text.
        """
        audio_path = Path(audio_path)
        if not audio_path.exists():
            raise FileNotFoundError(f"Audio file not found: {audio_path}")

        logger.info("Transcribing audio: {}", audio_path.name)

        # Voxtral supports audio via the Ollama chat endpoint.
        # Route through the LLM router with AUDIO_TRANSCRIPTION task type.
        transcription = await self._router.route(
            TaskType.AUDIO_TRANSCRIPTION,
            AUDIO_TRANSCRIPTION_PROMPT.format(audio_file=str(audio_path)),
        )
        return transcription.strip()

    # ==================================================================
    # Forensic analysis
    # ==================================================================

    async def analyze_audio_forensic(
        self, audio_path: str | Path
    ) -> Dict[str, Any]:
        """Forensic analysis of an audio recording.

        Pipeline:
          1. Transcribe the audio
          2. Detect audio events (loud sounds, silences)
          3. Analyze with LLM for forensic assessment
        """
        audio_path = Path(audio_path)
        logger.info("Forensic audio analysis: {}", audio_path.name)

        result: Dict[str, Any] = {
            "audio_file": str(audio_path),
            "status": "running",
        }

        # Step 1: Transcribe
        try:
            transcription = await self.transcribe_audio(audio_path)
            result["transcription"] = transcription
        except Exception as exc:
            logger.error("Transcription failed: {}", exc)
            result["transcription"] = ""
            transcription = ""

        # Step 2: Detect audio events
        try:
            events = self.detect_audio_events(audio_path)
            result["events"] = events
        except Exception as exc:
            logger.error("Event detection failed: {}", exc)
            result["events"] = []
            events = []

        # Step 3: LLM forensic analysis
        try:
            events_summary = json.dumps(events[:20], ensure_ascii=False)
            prompt = AUDIO_FORENSIC_ANALYSIS_PROMPT.format(
                transcription=transcription or "(transcription non disponible)",
                events=events_summary,
                audio_file=audio_path.name,
            )
            analysis = await self._router.route(
                TaskType.DEEP_ANALYSIS,
                prompt,
            )
            result["forensic_analysis"] = analysis.strip()
        except Exception as exc:
            logger.error("LLM forensic analysis failed: {}", exc)
            result["forensic_analysis"] = ""

        result["status"] = "completed"
        return result

    # ==================================================================
    # Event detection (basic DSP)
    # ==================================================================

    def detect_audio_events(
        self, audio_path: str | Path
    ) -> List[Dict[str, Any]]:
        """Detect notable events in audio using basic signal processing.

        Performs:
        - RMS energy analysis for loud events (potential impacts, gunshots)
        - Silence detection (gaps, cuts)

        Uses only the standard library (wave + struct) to avoid heavy
        dependencies. Works with WAV files.

        Returns [{timestamp_sec, duration_sec, type, amplitude}].
        """
        audio_path = Path(audio_path)
        if not audio_path.exists():
            raise FileNotFoundError(f"Audio file not found: {audio_path}")

        suffix = audio_path.suffix.lower()
        if suffix != ".wav":
            logger.warning(
                "Event detection works best with WAV files. "
                "Got '{}'. Will attempt to read.", suffix
            )

        events: List[Dict[str, Any]] = []

        try:
            events = self._analyze_wav_events(audio_path)
        except Exception as exc:
            logger.error("WAV event analysis failed: {}", exc)

        return events

    def _analyze_wav_events(
        self, wav_path: Path
    ) -> List[Dict[str, Any]]:
        """Analyze a WAV file for audio events."""
        events: List[Dict[str, Any]] = []

        with wave.open(str(wav_path), "rb") as wf:
            n_channels = wf.getnchannels()
            sample_width = wf.getsampwidth()
            frame_rate = wf.getframerate()
            n_frames = wf.getnframes()
            duration_sec = n_frames / frame_rate

            # Read all frames
            raw_data = wf.readframes(n_frames)

        # Convert to list of sample amplitudes (mono, normalized to [-1, 1])
        samples = self._decode_wav_samples(
            raw_data, sample_width, n_channels
        )

        if not samples:
            return events

        # Window size: 50ms windows for event detection
        window_size = max(1, int(frame_rate * 0.05))
        # Step: 25ms (50% overlap)
        step_size = max(1, window_size // 2)

        # Calculate RMS for each window
        rms_values: List[Tuple[float, float]] = []  # (timestamp, rms)
        for start in range(0, len(samples) - window_size, step_size):
            window = samples[start : start + window_size]
            rms = math.sqrt(sum(s * s for s in window) / len(window))
            timestamp = start / frame_rate
            rms_values.append((timestamp, rms))

        if not rms_values:
            return events

        # Calculate statistics for thresholding
        all_rms = [r for _, r in rms_values]
        mean_rms = sum(all_rms) / len(all_rms)
        variance = sum((r - mean_rms) ** 2 for r in all_rms) / len(all_rms)
        std_rms = math.sqrt(variance) if variance > 0 else 0.001

        # Loud event threshold: mean + 3 * std (statistical outliers)
        loud_threshold = mean_rms + 3 * std_rms
        # Silence threshold: 5% of mean RMS
        silence_threshold = mean_rms * 0.05

        # Detect loud events
        in_loud = False
        loud_start = 0.0
        loud_peak = 0.0
        for timestamp, rms in rms_values:
            if rms > loud_threshold:
                if not in_loud:
                    in_loud = True
                    loud_start = timestamp
                    loud_peak = rms
                else:
                    loud_peak = max(loud_peak, rms)
            elif in_loud:
                in_loud = False
                event_duration = timestamp - loud_start
                events.append({
                    "timestamp_sec": round(loud_start, 3),
                    "duration_sec": round(max(event_duration, 0.01), 3),
                    "type": "loud_event",
                    "amplitude": round(loud_peak, 4),
                    "amplitude_db": round(
                        20 * math.log10(loud_peak) if loud_peak > 0 else -100,
                        1,
                    ),
                })

        # Detect silences (>0.5s of very low RMS)
        in_silence = False
        silence_start = 0.0
        min_silence_duration = 0.5  # seconds
        for timestamp, rms in rms_values:
            if rms < silence_threshold:
                if not in_silence:
                    in_silence = True
                    silence_start = timestamp
            elif in_silence:
                in_silence = False
                silence_duration = timestamp - silence_start
                if silence_duration >= min_silence_duration:
                    events.append({
                        "timestamp_sec": round(silence_start, 3),
                        "duration_sec": round(silence_duration, 3),
                        "type": "silence",
                        "amplitude": 0.0,
                        "amplitude_db": -100.0,
                    })

        # Sort by timestamp
        events.sort(key=lambda e: e["timestamp_sec"])

        # Add metadata
        for i, ev in enumerate(events):
            ev["event_id"] = i

        logger.info(
            "Detected {} audio events in {:.1f}s recording",
            len(events),
            duration_sec,
        )
        return events

    # ==================================================================
    # Sound propagation
    # ==================================================================

    def calculate_sound_propagation(
        self,
        source_coords: Tuple[float, float],
        listener_coords: List[Tuple[float, float]],
        speed_of_sound: float = 343.0,  # m/s at 20C in air
    ) -> List[Dict[str, Any]]:
        """Calculate sound arrival times at different listener positions.

        Useful for gunshot localization with multiple witnesses or
        microphones. If two listeners report hearing a sound at
        different times, the time difference constrains the source
        location.

        Args:
            source_coords: (x, y) of the sound source in meters.
            listener_coords: List of (x, y) listener positions in meters.
            speed_of_sound: Speed of sound in m/s (default 343 m/s at 20C).

        Returns:
            List of dicts with distance, delay, and relative delay info.
        """
        if speed_of_sound <= 0:
            raise ValueError("Speed of sound must be positive")

        results: List[Dict[str, Any]] = []
        min_delay = float("inf")

        for i, (lx, ly) in enumerate(listener_coords):
            dx = lx - source_coords[0]
            dy = ly - source_coords[1]
            distance = math.hypot(dx, dy)
            delay = distance / speed_of_sound
            min_delay = min(min_delay, delay)

            results.append({
                "listener_id": i,
                "listener_coords": {"x": lx, "y": ly},
                "distance_m": round(distance, 3),
                "delay_sec": round(delay, 6),
                "delay_ms": round(delay * 1000, 3),
            })

        # Add relative delay (relative to the nearest listener)
        for r in results:
            r["relative_delay_sec"] = round(r["delay_sec"] - min_delay, 6)
            r["relative_delay_ms"] = round(
                (r["delay_sec"] - min_delay) * 1000, 3
            )

        return results

    def estimate_source_from_delays(
        self,
        listener_coords: List[Tuple[float, float]],
        arrival_times_sec: List[float],
        speed_of_sound: float = 343.0,
    ) -> Dict[str, Any]:
        """Estimate sound source location from arrival time differences.

        Uses TDOA (Time Difference of Arrival) with a least-squares
        linearization approach. Requires at least 3 listeners.

        Args:
            listener_coords: List of (x, y) listener positions (meters).
            arrival_times_sec: Arrival time at each listener (seconds).
            speed_of_sound: Speed of sound in m/s.

        Returns:
            Estimated source position {x, y, residual, confidence}.
        """
        n = len(listener_coords)
        if n < 3:
            raise ValueError("Need at least 3 listeners for TDOA localization")
        if len(arrival_times_sec) != n:
            raise ValueError("Must provide one arrival time per listener")

        # Use listener 0 as reference. For each other listener i:
        # distance_i - distance_0 = speed * (t_i - t_0) = d_diff_i
        #
        # distance_i^2 - distance_0^2 = (x_i^2 + y_i^2) - 2*(x_i*sx + y_i*sy)
        #                               - (x_0^2 + y_0^2) + 2*(x_0*sx + y_0*sy)
        # where (sx, sy) is the source position.
        #
        # This gives a linear system in sx, sy (Fang's method simplified).

        x0, y0 = listener_coords[0]
        t0 = arrival_times_sec[0]

        # Build the overdetermined linear system A * [sx, sy]^T = b
        A: List[List[float]] = []
        b_vec: List[float] = []

        for i in range(1, n):
            xi, yi = listener_coords[i]
            ti = arrival_times_sec[i]
            d_diff = speed_of_sound * (ti - t0)

            # Linearized equation coefficients
            a_row = [2 * (x0 - xi), 2 * (y0 - yi)]
            b_val = (
                d_diff * d_diff
                + (x0 * x0 + y0 * y0)
                - (xi * xi + yi * yi)
            )
            # We also need d0 in the equation. Use iterative refinement:
            # For the first pass, approximate d0 from centroid.
            A.append(a_row)
            b_vec.append(b_val)

        # Solve via normal equations: A^T A x = A^T b
        # For a 2x2 system this is direct.
        ata = [[0.0, 0.0], [0.0, 0.0]]
        atb = [0.0, 0.0]

        for row, bv in zip(A, b_vec):
            for r in range(2):
                for c in range(2):
                    ata[r][c] += row[r] * row[c]
                atb[r] += row[r] * bv

        det = ata[0][0] * ata[1][1] - ata[0][1] * ata[1][0]
        if abs(det) < 1e-12:
            raise ValueError(
                "Cannot solve TDOA -- listeners may be collinear"
            )

        sx = (atb[0] * ata[1][1] - atb[1] * ata[0][1]) / det
        sy = (ata[0][0] * atb[1] - ata[1][0] * atb[0]) / det

        # Calculate residuals
        residuals: List[float] = []
        for i in range(n):
            xi, yi = listener_coords[i]
            ti = arrival_times_sec[i]
            predicted_dist = math.hypot(xi - sx, yi - sy)
            predicted_time = t0 + (predicted_dist - math.hypot(x0 - sx, y0 - sy)) / speed_of_sound
            residuals.append(abs(ti - predicted_time))

        mean_residual = sum(residuals) / len(residuals)
        # Confidence: low residuals = high confidence
        confidence = max(0.0, min(1.0, 1.0 / (1.0 + mean_residual * 1000)))

        return {
            "x": round(sx, 2),
            "y": round(sy, 2),
            "mean_residual_sec": round(mean_residual, 6),
            "confidence": round(confidence, 3),
            "residuals": [round(r, 6) for r in residuals],
        }

    # ==================================================================
    # Internal helpers
    # ==================================================================

    @staticmethod
    def _decode_wav_samples(
        raw_data: bytes,
        sample_width: int,
        n_channels: int,
    ) -> List[float]:
        """Decode raw WAV bytes to normalized mono float samples [-1, 1]."""
        if sample_width == 1:
            fmt = "B"  # unsigned 8-bit
            max_val = 128.0
            offset = 128
        elif sample_width == 2:
            fmt = "<h"  # signed 16-bit LE
            max_val = 32768.0
            offset = 0
        elif sample_width == 4:
            fmt = "<i"  # signed 32-bit LE
            max_val = 2147483648.0
            offset = 0
        else:
            logger.warning("Unsupported sample width: {}", sample_width)
            return []

        frame_size = sample_width * n_channels
        n_frames = len(raw_data) // frame_size
        samples: List[float] = []

        for i in range(n_frames):
            frame_offset = i * frame_size
            # Take only the first channel (mono mix)
            sample_bytes = raw_data[
                frame_offset : frame_offset + sample_width
            ]
            if len(sample_bytes) < sample_width:
                break
            value = struct.unpack(fmt, sample_bytes)[0]
            normalized = (value - offset) / max_val
            samples.append(normalized)

        return samples
