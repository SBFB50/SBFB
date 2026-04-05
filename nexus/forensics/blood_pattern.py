"""
NEXUS -- Blood Pattern Analysis (BPA).

Computational analysis of bloodstain patterns using:
- VLM (gemma4/qwen3-vl) for pattern classification from photos
- Geometric calculations for angle of impact and area of convergence
- LLM reasoning for interpretation in investigation context
"""

from __future__ import annotations

import json
import math
import re
from pathlib import Path
from typing import Any, Dict, List, Optional

from loguru import logger

from nexus.llm.prompts import (
    BPA_CLASSIFICATION_PROMPT,
    BPA_INTERPRETATION_PROMPT,
    BPA_SPATTER_ANALYSIS_PROMPT,
)
from nexus.llm.router import LLMRouter, TaskType


class BloodPatternAnalyzer:
    """Analyze bloodstain patterns from crime scene photos and measurements."""

    def __init__(self, router: LLMRouter) -> None:
        self._router = router

    # ==================================================================
    # VLM-based classification
    # ==================================================================

    async def classify_pattern(self, image_path: str | Path) -> Dict[str, Any]:
        """Classify a bloodstain pattern from a photo using VLM.

        Returns classification, description, and forensic implications.
        Pattern types: spatter, transfer, drip, pool, cast-off,
        arterial spurt, expirated, void, swipe, wipe, saturation.
        """
        logger.info("BPA classification: {}", Path(image_path).name)
        raw = await self._router.route_vision(
            TaskType.IMAGE_SCENE_ANALYSIS,
            BPA_CLASSIFICATION_PROMPT,
            image_path,
        )
        return self._parse_json_response(raw, fallback_key="classification")

    # ==================================================================
    # Detailed spatter analysis
    # ==================================================================

    async def analyze_spatter(self, image_path: str | Path) -> Dict[str, Any]:
        """Deep analysis of a blood spatter pattern.

        Identifies individual stains, estimates angles of impact,
        determines area of convergence and area of origin.
        """
        logger.info("BPA spatter analysis: {}", Path(image_path).name)
        raw = await self._router.route_vision(
            TaskType.IMAGE_SCENE_ANALYSIS,
            BPA_SPATTER_ANALYSIS_PROMPT,
            image_path,
        )
        return self._parse_json_response(raw, fallback_key="spatter_analysis")

    # ==================================================================
    # Geometric calculations
    # ==================================================================

    def calculate_impact_angle(self, width: float, length: float) -> float:
        """Calculate angle of impact from stain dimensions.

        Uses: sin(angle) = width / length
        where width is the minor axis and length is the major axis
        of the elliptical stain.

        Returns angle in degrees (0-90). A circular stain (w==l)
        gives 90 degrees (perpendicular impact).
        """
        if length <= 0 or width <= 0:
            raise ValueError(
                f"Dimensions must be positive (width={width}, length={length})"
            )
        if width > length:
            raise ValueError(
                f"Width ({width}) cannot exceed length ({length})"
            )
        ratio = width / length
        # Clamp to avoid floating-point issues at boundary
        ratio = min(ratio, 1.0)
        return math.degrees(math.asin(ratio))

    def calculate_area_of_convergence(
        self, stains: List[Dict[str, Any]]
    ) -> Dict[str, Any]:
        """Calculate 2D area of convergence from multiple stain measurements.

        Each stain dict must contain:
          - x: float -- x position of stain on surface (mm or consistent unit)
          - y: float -- y position of stain on surface
          - direction_degrees: float -- direction the blood *came from*
            (0 = right/+x, 90 = up/+y, counter-clockwise)

        Uses least-squares intersection of directional lines from each
        stain to find the best-fit convergence point.

        Returns {center_x, center_y, radius, confidence, intersections}.
        Requires at least 2 stains.
        """
        if len(stains) < 2:
            raise ValueError("Need at least 2 stains to compute convergence")

        # Build lines from each stain in the direction blood came from.
        # Each stain at (x, y) with direction theta gives a ray.
        # We convert each to the parametric form: ax + by = c
        # where (a, b) is the normal to the direction.
        #
        # Direction vector: (cos(theta), sin(theta))
        # Normal vector: (-sin(theta), cos(theta))
        # Line equation: -sin(theta)*X + cos(theta)*Y = -sin(theta)*x0 + cos(theta)*y0

        lines: List[tuple[float, float, float]] = []
        for s in stains:
            theta = math.radians(s["direction_degrees"])
            a = -math.sin(theta)
            b = math.cos(theta)
            c = a * s["x"] + b * s["y"]
            lines.append((a, b, c))

        # Find all pairwise intersection points
        intersections: List[tuple[float, float]] = []
        n = len(lines)
        for i in range(n):
            for j in range(i + 1, n):
                a1, b1, c1 = lines[i]
                a2, b2, c2 = lines[j]
                det = a1 * b2 - a2 * b1
                if abs(det) < 1e-10:
                    # Lines are nearly parallel -- skip
                    continue
                ix = (c1 * b2 - c2 * b1) / det
                iy = (a1 * c2 - a2 * c1) / det
                intersections.append((ix, iy))

        if not intersections:
            raise ValueError(
                "No valid intersections found -- lines may be parallel"
            )

        # Compute centroid of intersection points as convergence center
        cx = sum(p[0] for p in intersections) / len(intersections)
        cy = sum(p[1] for p in intersections) / len(intersections)

        # Compute radius as the standard deviation of distances from centroid
        distances = [
            math.hypot(p[0] - cx, p[1] - cy) for p in intersections
        ]
        mean_dist = sum(distances) / len(distances)
        if len(distances) > 1:
            variance = sum((d - mean_dist) ** 2 for d in distances) / (
                len(distances) - 1
            )
            radius = math.sqrt(variance)
        else:
            radius = 0.0

        # Confidence: inversely related to scatter of intersections.
        # Tight cluster = high confidence. Uses 1/(1 + normalized_radius).
        max_dim = max(
            max(abs(p[0] - cx) for p in intersections),
            max(abs(p[1] - cy) for p in intersections),
            1.0,
        )
        confidence = 1.0 / (1.0 + radius / max_dim)

        return {
            "center_x": round(cx, 2),
            "center_y": round(cy, 2),
            "radius": round(radius, 2),
            "confidence": round(confidence, 3),
            "num_intersections": len(intersections),
            "intersections": [
                {"x": round(p[0], 2), "y": round(p[1], 2)}
                for p in intersections
            ],
        }

    def estimate_area_of_origin(
        self, stains: List[Dict[str, Any]], convergence: Dict[str, Any]
    ) -> Dict[str, Any]:
        """Estimate 3D point of origin using the tangent method.

        Each stain dict must contain:
          - x, y: position on surface
          - angle_degrees: angle of impact (from calculate_impact_angle)

        Uses: tan(angle) = height / horizontal_distance_to_convergence

        The height (z) is estimated for each stain individually, then
        averaged (weighted by confidence from angle magnitude).

        Returns {x, y, z_height, individual_estimates, confidence}.
        """
        cx = convergence["center_x"]
        cy = convergence["center_y"]

        estimates: List[Dict[str, Any]] = []

        for i, s in enumerate(stains):
            angle_deg = s.get("angle_degrees", 0.0)
            if angle_deg <= 0 or angle_deg >= 90:
                # Skip invalid angles (0 = parallel, 90 = no horizontal info)
                continue

            # Horizontal distance from stain to convergence point
            dx = s["x"] - cx
            dy = s["y"] - cy
            horiz_dist = math.hypot(dx, dy)

            if horiz_dist < 1e-6:
                # Stain is at the convergence point
                continue

            angle_rad = math.radians(angle_deg)
            z_height = horiz_dist * math.tan(angle_rad)

            # Weight: stains with steeper angles give more reliable height
            # estimates. Weight by sin(angle) since near-0 angles amplify error.
            weight = math.sin(angle_rad)

            estimates.append({
                "stain_index": i,
                "horizontal_distance": round(horiz_dist, 2),
                "angle_degrees": round(angle_deg, 2),
                "z_height": round(z_height, 2),
                "weight": round(weight, 3),
            })

        if not estimates:
            raise ValueError(
                "No valid height estimates -- check stain angles"
            )

        # Weighted average of z-height
        total_weight = sum(e["weight"] for e in estimates)
        z_avg = sum(e["z_height"] * e["weight"] for e in estimates) / total_weight

        # Confidence based on consistency of estimates
        if len(estimates) > 1:
            z_values = [e["z_height"] for e in estimates]
            z_mean = sum(z_values) / len(z_values)
            z_variance = sum((z - z_mean) ** 2 for z in z_values) / (
                len(z_values) - 1
            )
            z_std = math.sqrt(z_variance)
            # Coefficient of variation (lower = more consistent)
            cv = z_std / abs(z_avg) if abs(z_avg) > 1e-6 else 1.0
            confidence = max(0.0, min(1.0, 1.0 / (1.0 + cv)))
        else:
            confidence = 0.5  # Single estimate -- moderate confidence

        return {
            "x": convergence["center_x"],
            "y": convergence["center_y"],
            "z_height": round(z_avg, 2),
            "confidence": round(confidence, 3),
            "num_estimates": len(estimates),
            "individual_estimates": estimates,
        }

    # ==================================================================
    # Full BPA pipeline
    # ==================================================================

    async def full_bpa_analysis(
        self,
        image_path: str | Path,
        measurements: Optional[List[Dict[str, Any]]] = None,
        case_context: str = "",
    ) -> Dict[str, Any]:
        """Complete BPA analysis combining VLM and calculations.

        Pipeline:
          1. VLM classifies the pattern type
          2. VLM identifies individual stains and estimates dimensions
          3. Calculate angles of impact (if measurements provided)
          4. Calculate area of convergence (if multiple stains)
          5. Estimate area of origin
          6. LLM interprets findings in case context

        Returns comprehensive analysis dict.
        """
        image_path = Path(image_path)
        logger.info("Full BPA analysis: {}", image_path.name)

        result: Dict[str, Any] = {
            "image": str(image_path),
            "status": "running",
        }

        # --- Step 1: Classify pattern ---
        try:
            classification = await self.classify_pattern(image_path)
            result["classification"] = classification
            logger.info("BPA pattern classified")
        except Exception as exc:
            logger.error("BPA classification failed: {}", exc)
            result["classification"] = {"error": str(exc)}

        # --- Step 2: Spatter analysis ---
        try:
            spatter = await self.analyze_spatter(image_path)
            result["spatter_analysis"] = spatter
            logger.info("BPA spatter analysis complete")
        except Exception as exc:
            logger.error("BPA spatter analysis failed: {}", exc)
            result["spatter_analysis"] = {"error": str(exc)}

        # --- Steps 3-5: Geometric calculations (if measurements) ---
        if measurements:
            # Step 3: Calculate angles
            angles: List[Dict[str, Any]] = []
            for i, m in enumerate(measurements):
                try:
                    w = float(m.get("width", 0))
                    l = float(m.get("length", 0))
                    angle = self.calculate_impact_angle(w, l)
                    angles.append({
                        "stain_index": i,
                        "width": w,
                        "length": l,
                        "impact_angle": round(angle, 2),
                    })
                    # Enrich the measurement with the calculated angle
                    m["angle_degrees"] = angle
                except (ValueError, KeyError) as exc:
                    logger.warning("Skipping stain {}: {}", i, exc)
                    angles.append({
                        "stain_index": i,
                        "error": str(exc),
                    })
            result["impact_angles"] = angles

            # Step 4: Area of convergence (need at least 2 stains with direction)
            stains_with_direction = [
                m for m in measurements if "direction_degrees" in m
            ]
            if len(stains_with_direction) >= 2:
                try:
                    convergence = self.calculate_area_of_convergence(
                        stains_with_direction
                    )
                    result["convergence"] = convergence

                    # Step 5: Area of origin (need angles + convergence)
                    stains_with_angles = [
                        m
                        for m in stains_with_direction
                        if "angle_degrees" in m and m["angle_degrees"] > 0
                    ]
                    if stains_with_angles:
                        try:
                            origin = self.estimate_area_of_origin(
                                stains_with_angles, convergence
                            )
                            result["area_of_origin"] = origin
                        except ValueError as exc:
                            logger.warning("Origin estimation failed: {}", exc)
                            result["area_of_origin"] = {"error": str(exc)}
                except ValueError as exc:
                    logger.warning("Convergence calculation failed: {}", exc)
                    result["convergence"] = {"error": str(exc)}

        # --- Step 6: LLM interpretation ---
        try:
            interpretation = await self._interpret_findings(
                result, case_context
            )
            result["interpretation"] = interpretation
        except Exception as exc:
            logger.error("BPA interpretation failed: {}", exc)
            result["interpretation"] = {"error": str(exc)}

        result["status"] = "completed"
        return result

    # ==================================================================
    # Internal helpers
    # ==================================================================

    async def _interpret_findings(
        self, findings: Dict[str, Any], case_context: str
    ) -> str:
        """Use LLM to interpret BPA findings in the investigation context."""
        # Build a text summary of findings for the LLM
        findings_text = json.dumps(findings, indent=2, ensure_ascii=False)

        prompt = BPA_INTERPRETATION_PROMPT.format(
            findings=findings_text,
            case_context=case_context or "Aucun contexte fourni.",
        )
        interpretation = await self._router.route(
            TaskType.DEEP_ANALYSIS,
            prompt,
        )
        return interpretation.strip()

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
            # Try to find a JSON object in the text
            match = re.search(r"\{[\s\S]*\}", cleaned)
            if match:
                try:
                    return json.loads(match.group())
                except json.JSONDecodeError:
                    pass
            logger.warning("Failed to parse BPA JSON, returning raw text")
            return {fallback_key: raw.strip()}
