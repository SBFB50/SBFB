"""
NEXUS -- Physics simulation API router.

Endpoints for forensic physics simulations:
- Blood drop trajectory simulation
- Cast-off pattern simulation
- Sound propagation modelling
- The Well dataset listing
- Origin-of-impact estimation
"""

from __future__ import annotations

from typing import Any, Dict, List, Optional

from fastapi import APIRouter, HTTPException
from loguru import logger
from pydantic import BaseModel, Field

from nexus.forensics.physics_sim import ForensicPhysicsSim
from nexus.forensics.the_well_loader import TheWellLoader


router = APIRouter(prefix="/api/forensics/sim", tags=["physics-sim"])


# -----------------------------------------------------------------------
# Singletons (stateless, safe to share across requests)
# -----------------------------------------------------------------------

_sim = ForensicPhysicsSim()
_well = TheWellLoader()


# -----------------------------------------------------------------------
# Request / response models
# -----------------------------------------------------------------------

class BloodDropRequest(BaseModel):
    velocity: float = Field(..., gt=0, description="Release speed in m/s")
    angle: float = Field(..., ge=-90, le=90, description="Launch angle in degrees from horizontal")
    height: float = Field(..., gt=0, description="Release height above surface in metres")
    surface_angle: float = Field(0.0, ge=0, le=90, description="Surface tilt in degrees")
    blood_properties: Optional[Dict[str, float]] = Field(
        None,
        description="Override blood properties: density, viscosity, surface_tension, drop_diameter",
    )


class CastOffRequest(BaseModel):
    swing_radius: float = Field(..., gt=0, description="Pivot to weapon tip distance in metres")
    swing_speed: float = Field(..., gt=0, description="Angular velocity in rad/s")
    num_drops: int = Field(20, ge=1, le=100, description="Number of drops to simulate")
    blood_on_weapon_length: float = Field(0.3, gt=0, description="Bloody section length in metres")
    swing_plane_height: float = Field(1.5, gt=0, description="Pivot height above floor in metres")
    swing_start_angle: float = Field(-30.0, description="Start of swing arc in degrees")
    swing_end_angle: float = Field(150.0, description="End of swing arc in degrees")
    blood_properties: Optional[Dict[str, float]] = None


class SoundRequest(BaseModel):
    source: List[float] = Field(..., min_length=3, max_length=3, description="(x, y, z) metres")
    listeners: List[List[float]] = Field(
        ..., min_length=1,
        description="List of (x, y, z) listener positions",
    )
    source_db: float = Field(160.0, ge=0, le=200, description="Source level dB SPL at 1 m")
    frequency: float = Field(2000.0, gt=0, description="Dominant frequency in Hz")
    temperature: float = Field(20.0, ge=-40, le=60, description="Temperature in Celsius")
    humidity: float = Field(50.0, ge=0, le=100, description="Relative humidity percent")
    wind_speed: float = Field(0.0, ge=0, description="Wind speed in m/s")
    wind_direction: float = Field(0.0, ge=0, le=360, description="Wind direction in degrees from north")
    terrain: str = Field("urban", pattern="^(urban|rural|indoor)$")


class StainMeasurement(BaseModel):
    x: float = Field(..., description="X position on surface in metres")
    y: float = Field(..., description="Y position on surface in metres")
    width_mm: float = Field(..., gt=0, description="Stain minor axis in mm")
    length_mm: float = Field(..., gt=0, description="Stain major axis in mm")
    direction: float = Field(..., ge=0, lt=360, description="Travel direction in degrees (0=north)")


class OriginEstimationRequest(BaseModel):
    stains: List[StainMeasurement] = Field(
        ..., min_length=2,
        description="At least 2 stain measurements",
    )


# -----------------------------------------------------------------------
# POST /api/forensics/sim/blood-drop
# -----------------------------------------------------------------------

@router.post("/blood-drop")
async def simulate_blood_drop(req: BloodDropRequest) -> Dict[str, Any]:
    """Simulate a single blood drop trajectory and impact pattern.

    Computes projectile motion with Reynolds-dependent drag,
    and estimates the resulting elliptical stain geometry.
    """
    logger.info(
        "Blood drop sim: v={} m/s, angle={} deg, h={} m",
        req.velocity, req.angle, req.height,
    )
    try:
        result = _sim.simulate_blood_drop(
            velocity=req.velocity,
            angle=req.angle,
            height=req.height,
            blood_properties=req.blood_properties,
            surface_angle=req.surface_angle,
        )
        return result
    except Exception as exc:
        logger.error("Blood drop simulation failed: {}", exc)
        raise HTTPException(status_code=500, detail=str(exc))


# -----------------------------------------------------------------------
# POST /api/forensics/sim/cast-off
# -----------------------------------------------------------------------

@router.post("/cast-off")
async def simulate_cast_off(req: CastOffRequest) -> Dict[str, Any]:
    """Simulate a cast-off blood pattern from a swinging weapon.

    Models droplet detachment along the swing arc based on
    centripetal force vs. surface tension adhesion.
    """
    logger.info(
        "Cast-off sim: radius={} m, omega={} rad/s, drops={}",
        req.swing_radius, req.swing_speed, req.num_drops,
    )
    try:
        drops = _sim.simulate_cast_off(
            swing_radius=req.swing_radius,
            swing_speed=req.swing_speed,
            num_drops=req.num_drops,
            blood_on_weapon_length=req.blood_on_weapon_length,
            swing_plane_height=req.swing_plane_height,
            swing_start_angle=req.swing_start_angle,
            swing_end_angle=req.swing_end_angle,
            blood_properties=req.blood_properties,
        )
        return {
            "num_drops_released": len(drops),
            "num_drops_requested": req.num_drops,
            "drops": drops,
        }
    except Exception as exc:
        logger.error("Cast-off simulation failed: {}", exc)
        raise HTTPException(status_code=500, detail=str(exc))


# -----------------------------------------------------------------------
# POST /api/forensics/sim/sound
# -----------------------------------------------------------------------

@router.post("/sound")
async def simulate_sound_propagation(req: SoundRequest) -> Dict[str, Any]:
    """Simulate sound propagation from a point source to multiple listeners.

    Accounts for geometric spreading, atmospheric absorption (ISO 9613),
    terrain effects, and wind.
    """
    logger.info(
        "Sound sim: source={}, {} listeners, terrain={}",
        req.source, len(req.listeners), req.terrain,
    )
    try:
        # Validate listener positions
        listeners_tuples = []
        for i, pos in enumerate(req.listeners):
            if len(pos) != 3:
                raise HTTPException(
                    status_code=422,
                    detail=f"Listener {i} must have exactly 3 coordinates (x, y, z)",
                )
            listeners_tuples.append(tuple(pos))

        result = _sim.simulate_sound_propagation(
            source=tuple(req.source),
            listeners=listeners_tuples,
            source_db=req.source_db,
            frequency=req.frequency,
            temperature=req.temperature,
            humidity=req.humidity,
            wind_speed=req.wind_speed,
            wind_direction=req.wind_direction,
            terrain=req.terrain,
        )
        return result
    except HTTPException:
        raise
    except Exception as exc:
        logger.error("Sound propagation simulation failed: {}", exc)
        raise HTTPException(status_code=500, detail=str(exc))


# -----------------------------------------------------------------------
# POST /api/forensics/sim/origin
# -----------------------------------------------------------------------

@router.post("/origin")
async def estimate_origin(req: OriginEstimationRequest) -> Dict[str, Any]:
    """Estimate the area of origin from bloodstain measurements.

    Uses the tangent method (arcsin of width/length) to project
    convergence lines back to a common origin point.
    """
    logger.info("Origin estimation from {} stains", len(req.stains))
    try:
        stains_dicts = [s.model_dump() for s in req.stains]
        result = _sim.estimate_origin_of_impact(stains_dicts)
        if "error" in result:
            raise HTTPException(status_code=422, detail=result["error"])
        return result
    except HTTPException:
        raise
    except Exception as exc:
        logger.error("Origin estimation failed: {}", exc)
        raise HTTPException(status_code=500, detail=str(exc))


# -----------------------------------------------------------------------
# GET /api/forensics/sim/datasets
# -----------------------------------------------------------------------

@router.get("/datasets")
async def list_datasets() -> Dict[str, Any]:
    """List physics simulation datasets from The Well relevant to forensics."""
    datasets = _well.list_relevant_datasets()
    return {
        "the_well_installed": _well.available,
        "datasets": datasets,
    }


# -----------------------------------------------------------------------
# GET /api/forensics/sim/datasets/{name}
# -----------------------------------------------------------------------

@router.get("/datasets/{name}")
async def get_dataset_info(name: str) -> Dict[str, Any]:
    """Get detailed info about a specific The Well dataset."""
    info = _well.get_dataset_info(name)
    if info is None:
        raise HTTPException(status_code=404, detail=f"Dataset '{name}' not found")
    return info
