"""
NEXUS -- Physical trace analysis.

VLM-powered analysis of physical traces: fingerprints, tool marks,
tire tracks, shoe prints, glass fractures, fabric, hair, fiber, etc.

Uses the qwen3-vl deep vision model for detailed analysis and the
fast gemma4:e4b model for quick classification.
"""

from __future__ import annotations

import json
import re
from pathlib import Path
from typing import Any, Dict, List, Optional

from loguru import logger

from nexus.llm.prompts import (
    TRACE_ANALYSIS_PROMPT,
    TRACE_COMPARISON_PROMPT,
)
from nexus.llm.router import LLMRouter, TaskType

# Valid trace types for classification
TRACE_TYPES = frozenset({
    "fingerprint",
    "tool_mark",
    "tire_track",
    "shoe_print",
    "glass_fracture",
    "fabric",
    "hair",
    "fiber",
    "auto",
})


class TraceAnalyzer:
    """VLM-powered analysis of physical forensic traces."""

    def __init__(self, router: LLMRouter) -> None:
        self._router = router

    # ==================================================================
    # Single trace analysis
    # ==================================================================

    async def analyze_trace(
        self,
        image_path: str | Path,
        trace_type: str = "auto",
    ) -> Dict[str, Any]:
        """Analyze a physical trace from a photo.

        Args:
            image_path: Path to the trace photograph.
            trace_type: Type hint for the trace. One of:
                fingerprint, tool_mark, tire_track, shoe_print,
                glass_fracture, fabric, hair, fiber, auto.
                'auto' lets the VLM determine the type.

        Returns:
            Dict with type, classification, description, characteristics,
            forensic_value, and recommendations.
        """
        image_path = Path(image_path)
        if not image_path.exists():
            raise FileNotFoundError(f"Image not found: {image_path}")

        if trace_type not in TRACE_TYPES:
            logger.warning(
                "Unknown trace type '{}', falling back to 'auto'",
                trace_type,
            )
            trace_type = "auto"

        logger.info(
            "Analyzing trace: {} (type={})", image_path.name, trace_type
        )

        prompt = TRACE_ANALYSIS_PROMPT.format(trace_type=trace_type)

        raw = await self._router.route_vision(
            TaskType.TRACE_ANALYSIS,
            prompt,
            image_path,
        )

        result = self._parse_json_response(raw, fallback_key="analysis")
        result["image"] = str(image_path)
        result["requested_type"] = trace_type
        return result

    # ==================================================================
    # Trace comparison
    # ==================================================================

    async def compare_traces(
        self,
        image_1: str | Path,
        image_2: str | Path,
    ) -> Dict[str, Any]:
        """Compare two trace images for similarity.

        Since VLMs process one image at a time, we analyze both
        images independently, then ask the reasoning model to compare
        the textual descriptions.

        Returns:
            Dict with similarity_score, matching_features,
            differing_features, conclusion, and individual analyses.
        """
        image_1 = Path(image_1)
        image_2 = Path(image_2)

        if not image_1.exists():
            raise FileNotFoundError(f"Image 1 not found: {image_1}")
        if not image_2.exists():
            raise FileNotFoundError(f"Image 2 not found: {image_2}")

        logger.info(
            "Comparing traces: {} vs {}", image_1.name, image_2.name
        )

        # Step 1: Analyze both traces independently
        analysis_1 = await self.analyze_trace(image_1)
        analysis_2 = await self.analyze_trace(image_2)

        # Step 2: Build comparison prompt with both analyses
        analysis_1_text = json.dumps(analysis_1, indent=2, ensure_ascii=False)
        analysis_2_text = json.dumps(analysis_2, indent=2, ensure_ascii=False)

        prompt = TRACE_COMPARISON_PROMPT.format(
            trace_1=analysis_1_text,
            trace_2=analysis_2_text,
        )

        # Use reasoning model for comparison (logical analysis)
        raw = await self._router.route(
            TaskType.LOGIC_VERIFICATION,
            prompt,
        )

        comparison = self._parse_json_response(raw, fallback_key="comparison")
        comparison["image_1"] = str(image_1)
        comparison["image_2"] = str(image_2)
        comparison["analysis_1"] = analysis_1
        comparison["analysis_2"] = analysis_2

        return comparison

    # ==================================================================
    # Internal helpers
    # ==================================================================

    @staticmethod
    def _parse_json_response(
        raw: str, fallback_key: str = "result"
    ) -> Dict[str, Any]:
        """Parse JSON from VLM response, handling markdown fences."""
        cleaned = raw.strip()
        if cleaned.startswith("```"):
            cleaned = re.sub(r"^```(?:json)?\s*", "", cleaned)
            cleaned = re.sub(r"\s*```\s*$", "", cleaned)

        try:
            return json.loads(cleaned)
        except json.JSONDecodeError:
            match = re.search(r"\{[\s\S]*\}", cleaned)
            if match:
                try:
                    return json.loads(match.group())
                except json.JSONDecodeError:
                    pass
            logger.warning("Failed to parse trace JSON, returning raw text")
            return {fallback_key: raw.strip()}
