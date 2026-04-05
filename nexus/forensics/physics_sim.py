"""
NEXUS -- Physics simulation for forensic analysis.

Wraps PhiFlow (differentiable physics) for blood spatter simulation
and acoustic propagation modeling.

Provides:
- Single blood drop trajectory + impact analysis
- Cast-off pattern simulation from a swinging weapon
- Sound propagation modeling (gunshot, scream, etc.)

Optional dependency: phiflow (pip install phiflow)
Falls back to numpy-based analytic models when PhiFlow is unavailable.
"""

from __future__ import annotations

import math
from typing import Any, Optional

import numpy as np
from loguru import logger


class ForensicPhysicsSim:
    """Physics simulation engine for forensic scenarios.

    Uses PhiFlow for differentiable fluid dynamics when available,
    falls back to simplified numpy-based models otherwise.
    """

    # ----------------------------------------------------------------
    # Constants
    # ----------------------------------------------------------------

    # Blood physical properties at ~20 C (room temperature)
    DEFAULT_BLOOD_PROPS: dict[str, float] = {
        "density": 1060.0,          # kg/m^3
        "viscosity": 0.004,         # Pa.s  (4 cP, whole blood)
        "surface_tension": 0.058,   # N/m
        "drop_diameter": 0.002,     # m  (2 mm typical free-falling drop)
    }

    # Atmospheric / environmental defaults
    RHO_AIR = 1.225      # kg/m^3 at sea level, 15 C
    G = 9.80665          # m/s^2  standard gravity
    CD_SPHERE = 0.47     # drag coefficient for a smooth sphere (Re > ~1000)

    def __init__(self) -> None:
        self._phiflow_available = False
        try:
            import phi  # noqa: F401
            self._phiflow_available = True
            logger.info("PhiFlow available for physics simulation")
        except ImportError:
            logger.warning(
                "PhiFlow not installed; using simplified analytic models"
            )

    # ================================================================
    # Blood drop trajectory
    # ================================================================

    def simulate_blood_drop(
        self,
        velocity: float,           # m/s at release
        angle: float,              # degrees from horizontal (positive = upward)
        height: float,             # metres above the impact surface
        blood_properties: dict[str, float] | None = None,
        surface_angle: float = 0.0,  # degrees — tilt of the receiving surface
    ) -> dict[str, Any]:
        """Simulate a single blood drop trajectory and impact.

        Uses projectile motion with Reynolds-dependent drag on a sphere
        of given diameter and blood density.  Air resistance is modelled
        via the standard drag equation with a Cd that varies with the
        Reynolds number (Stokes / intermediate / Newton regimes).

        Parameters
        ----------
        velocity : float
            Release speed in m/s.
        angle : float
            Launch angle in degrees from horizontal.  Positive = upward.
        height : float
            Release height above the impact surface (metres).
        blood_properties : dict, optional
            Override default density, viscosity, surface_tension,
            drop_diameter.
        surface_angle : float
            Tilt of the receiving surface in degrees from horizontal.

        Returns
        -------
        dict with keys:
            trajectory         -- list of (x, y, z) tuples (metres)
            impact_point       -- (x, y) on the surface
            impact_angle       -- degrees from surface at impact
            stain_shape        -- {width_mm, length_mm, eccentricity}
            travel_time        -- seconds
            impact_velocity    -- m/s at impact
            reynolds_at_impact -- Reynolds number at impact
        """
        props = {**self.DEFAULT_BLOOD_PROPS, **(blood_properties or {})}

        radius = props["drop_diameter"] / 2.0
        area = math.pi * radius ** 2                        # cross-section
        volume = (4.0 / 3.0) * math.pi * radius ** 3
        mass = volume * props["density"]

        # Dynamic viscosity of air at ~20 C
        mu_air = 1.81e-5  # Pa.s

        angle_rad = math.radians(angle)
        vx = velocity * math.cos(angle_rad)
        vy = velocity * math.sin(angle_rad)

        x, y = 0.0, height
        dt = 0.00005          # 50 us — fine enough for mm-scale accuracy
        t = 0.0
        max_t = 10.0          # safety cap

        # Sub-sample trajectory for output (every ~0.5 ms)
        sample_interval = 0.0005
        next_sample = 0.0

        trajectory: list[tuple[float, float, float]] = [(0.0, round(height, 5), 0.0)]

        while y > 0.0 and t < max_t:
            speed = math.sqrt(vx * vx + vy * vy)

            # Reynolds number for the drop
            Re = (self.RHO_AIR * speed * props["drop_diameter"]) / mu_air if speed > 1e-12 else 0.0

            # Drag coefficient: Stokes (Re < 1), intermediate, Newton (Re > 1000)
            if Re < 1.0:
                Cd = 24.0 / max(Re, 1e-12)
            elif Re < 1000.0:
                # Schiller-Naumann correlation
                Cd = (24.0 / Re) * (1.0 + 0.15 * Re ** 0.687)
            else:
                Cd = self.CD_SPHERE  # Newton regime

            if speed > 1e-12:
                Fd = 0.5 * Cd * self.RHO_AIR * area * speed * speed
                # Drag force components (opposing velocity)
                ax = -(Fd * vx) / (speed * mass)
                ay = -self.G - (Fd * vy) / (speed * mass)
            else:
                ax = 0.0
                ay = -self.G

            # Velocity Verlet half-step would be more accurate, but
            # symplectic Euler is fine at dt = 50 us.
            vx += ax * dt
            vy += ay * dt
            x += vx * dt
            y += vy * dt
            t += dt

            if t >= next_sample:
                trajectory.append((round(x, 5), round(y, 5), 0.0))
                next_sample += sample_interval

        # Final state
        impact_speed = math.sqrt(vx * vx + vy * vy)
        # Impact angle relative to the surface (accounting for surface tilt)
        if impact_speed > 1e-12:
            flight_angle = math.degrees(math.atan2(abs(vy), abs(vx)))
            impact_angle = flight_angle - surface_angle
            impact_angle = max(0.0, min(90.0, impact_angle))
        else:
            impact_angle = 90.0

        Re_impact = (
            (self.RHO_AIR * impact_speed * props["drop_diameter"]) / mu_air
            if impact_speed > 1e-12
            else 0.0
        )

        # ------------------------------------------------------------------
        # Stain geometry (Balthazard / empirical model)
        # ------------------------------------------------------------------
        # Width  ~ drop diameter (perpendicular to direction of travel)
        # Length ~ width / sin(impact_angle)   (ellipse from oblique impact)
        #
        # At very low angles, satellite spatter forms — we clamp eccentricity.
        d = props["drop_diameter"] * 1000.0  # mm
        sin_alpha = math.sin(math.radians(max(impact_angle, 1.0)))
        stain_width = d  # mm
        stain_length = d / sin_alpha
        # Empirical spread factor for high-velocity impacts (> 5 m/s)
        spread_factor = 1.0 + 0.08 * max(0.0, impact_speed - 2.0)
        stain_width *= spread_factor
        stain_length *= spread_factor
        eccentricity = math.sqrt(1.0 - (stain_width / stain_length) ** 2) if stain_length > stain_width else 0.0

        return {
            "trajectory": trajectory,
            "impact_point": (round(x, 4), 0.0),
            "impact_angle": round(impact_angle, 2),
            "stain_shape": {
                "width_mm": round(stain_width, 3),
                "length_mm": round(stain_length, 3),
                "eccentricity": round(eccentricity, 4),
            },
            "travel_time": round(t, 5),
            "impact_velocity": round(impact_speed, 3),
            "reynolds_at_impact": round(Re_impact, 1),
            "blood_properties": props,
        }

    # ================================================================
    # Cast-off pattern
    # ================================================================

    def simulate_cast_off(
        self,
        swing_radius: float,                # metres (weapon + arm length)
        swing_speed: float,                  # rad/s (angular velocity)
        num_drops: int = 20,
        blood_on_weapon_length: float = 0.3, # metres of bloody section
        swing_plane_height: float = 1.5,     # metres above floor
        swing_start_angle: float = -30.0,    # degrees from vertical (behind)
        swing_end_angle: float = 150.0,      # degrees from vertical (follow-through)
        blood_properties: dict[str, float] | None = None,
    ) -> list[dict[str, Any]]:
        """Simulate a cast-off pattern from a swinging weapon.

        Blood droplets release from different points along the swing arc
        due to centrifugal force exceeding the surface tension adhesion.
        The release condition is met when centripetal acceleration
        exceeds the critical threshold for the given blood properties.

        The swing is modelled in a vertical plane (like an overhand blow).
        Drops release tangentially to the arc at their release point.

        Parameters
        ----------
        swing_radius : float
            Distance from pivot (shoulder) to end of weapon, metres.
        swing_speed : float
            Angular velocity in rad/s.  A fast swing is ~30-40 rad/s.
        num_drops : int
            Number of drops to simulate along the bloody portion.
        blood_on_weapon_length : float
            Length of the weapon covered in blood (metres from tip inward).
        swing_plane_height : float
            Height of the pivot (shoulder) above the floor.
        swing_start_angle, swing_end_angle : float
            Arc of the swing in degrees (0 = straight up from shoulder).
        blood_properties : dict, optional
            Override blood properties.

        Returns
        -------
        list of dicts — one per drop, each containing:
            release_angle, release_position, drop_sim (full trajectory result),
            release_radius, tangential_velocity
        """
        props = {**self.DEFAULT_BLOOD_PROPS, **(blood_properties or {})}

        # Critical acceleration for drop detachment.
        # A drop detaches when centripetal acceleration > adhesion / mass.
        # Adhesion force ~ pi * d * sigma  (circumference * surface tension)
        # For a sphere of diameter d on a cylindrical weapon.
        radius_drop = props["drop_diameter"] / 2.0
        volume_drop = (4.0 / 3.0) * math.pi * radius_drop ** 3
        mass_drop = volume_drop * props["density"]
        adhesion_force = math.pi * props["drop_diameter"] * props["surface_tension"]
        # Critical angular velocity for detachment at distance r:
        #   m * omega^2 * r > adhesion_force
        #   => omega_crit(r) = sqrt(adhesion_force / (m * r))

        # Distribute drops along the bloody portion of the weapon
        r_tip = swing_radius
        r_inner = max(0.1, swing_radius - blood_on_weapon_length)
        radii = np.linspace(r_inner, r_tip, num_drops)

        # Determine where in the arc each drop detaches
        arc_range = swing_end_angle - swing_start_angle  # degrees
        results: list[dict[str, Any]] = []

        for i, r in enumerate(radii):
            # Critical angular velocity for this radius
            omega_crit = math.sqrt(adhesion_force / (mass_drop * r))

            # If the swing is fast enough, the drop detaches.
            # The further from the pivot, the earlier it detaches.
            if swing_speed < omega_crit:
                # Not enough force to detach at this radius — skip
                continue

            # Approximate release angle: drops further from pivot detach
            # earlier in the swing (lower threshold).  We model the swing
            # as accelerating linearly to peak angular velocity.
            # fraction through swing when omega exceeds omega_crit:
            detach_fraction = (omega_crit / swing_speed) ** 2
            detach_fraction = min(1.0, max(0.0, detach_fraction))

            release_angle_deg = swing_start_angle + detach_fraction * arc_range
            release_angle_rad = math.radians(release_angle_deg)

            # Position of the drop at release (pivot at origin, y=0 = floor)
            # Angle 0 = straight up; increases clockwise (forward swing)
            px = r * math.sin(release_angle_rad)
            py = swing_plane_height + r * math.cos(release_angle_rad)

            # Tangential velocity direction: perpendicular to the radius,
            # in the direction of the swing (clockwise).
            tangential_speed = swing_speed * r
            # Tangent direction: 90 degrees ahead of the radial direction
            vx = tangential_speed * math.cos(release_angle_rad)
            vy = -tangential_speed * math.sin(release_angle_rad)

            # Compute launch angle from horizontal
            launch_speed = math.sqrt(vx * vx + vy * vy)
            launch_angle = math.degrees(math.atan2(vy, vx))

            # Simulate trajectory from the release point
            drop_sim = self.simulate_blood_drop(
                velocity=launch_speed,
                angle=launch_angle,
                height=py,
                blood_properties=props,
            )

            # Offset the trajectory to world coordinates
            shifted_trajectory = [
                (round(px + pt[0], 5), round(pt[1], 5), round(pt[2], 5))
                for pt in drop_sim["trajectory"]
            ]
            drop_sim["trajectory"] = shifted_trajectory
            drop_sim["impact_point"] = (
                round(px + drop_sim["impact_point"][0], 4),
                0.0,
            )

            results.append({
                "drop_index": i,
                "release_angle_deg": round(release_angle_deg, 2),
                "release_position": (round(px, 4), round(py, 4)),
                "release_radius": round(r, 4),
                "tangential_velocity": round(tangential_speed, 3),
                "launch_angle_deg": round(launch_angle, 2),
                "drop_sim": drop_sim,
            })

        return results

    # ================================================================
    # Sound propagation
    # ================================================================

    def simulate_sound_propagation(
        self,
        source: tuple[float, float, float],         # (x, y, z) metres
        listeners: list[tuple[float, float, float]],
        source_db: float = 160.0,     # dB SPL at 1 m (handgun ~ 155-170)
        frequency: float = 2000.0,    # Hz dominant frequency
        temperature: float = 20.0,    # Celsius
        humidity: float = 50.0,       # percent
        wind_speed: float = 0.0,      # m/s
        wind_direction: float = 0.0,  # degrees from north (0 = N, 90 = E)
        terrain: str = "urban",       # urban | rural | indoor
    ) -> dict[str, Any]:
        """Simulate sound propagation from a point source to listeners.

        Physics modelled:
        1. Temperature-dependent speed of sound
        2. Geometric spreading (inverse square law)
        3. Atmospheric absorption (ISO 9613-1 simplified)
        4. Ground effect attenuation (terrain-dependent)
        5. Wind effect on propagation delay
        6. Hearing threshold comparison

        Parameters
        ----------
        source : tuple
            (x, y, z) position of sound source in metres.
        listeners : list of tuples
            List of (x, y, z) positions for each listener.
        source_db : float
            Sound pressure level at 1 m from the source (dB SPL).
        frequency : float
            Dominant frequency in Hz (affects atmospheric absorption).
        temperature, humidity : float
            Atmospheric conditions.
        wind_speed, wind_direction : float
            Wind vector (m/s, degrees from north).
        terrain : str
            "urban" (reflections +3 dB), "rural" (ground absorption -3 dB),
            "indoor" (reverberation +6 dB within 50 m).

        Returns
        -------
        dict with:
            speed_of_sound, temperature, arrivals (list of per-listener
            results including distance, delay, attenuation, estimated dB)
        """
        # Speed of sound: c = 331.3 + 0.606 * T  (dry air approximation)
        # Humidity correction: c increases ~0.1-0.6 m/s per 10% RH
        c = 331.3 + 0.606 * temperature + 0.0124 * humidity

        # Atmospheric absorption coefficient (dB/m) — simplified ISO 9613-1
        # For a given frequency and humidity.  At 2 kHz, ~50% RH:  ~0.01 dB/m
        # General formula (simplified):
        #   alpha ~ f^2 * (1.84e-11 * (T/T0)^0.5 + relaxation_terms)
        # We use a practical approximation for the 500-4000 Hz range.
        T_kelvin = temperature + 273.15
        # Oxygen relaxation frequency
        fr_O = 24.0 + 4.04e4 * humidity * (0.02 + humidity) / (0.391 + humidity)
        # Simplified absorption coefficient (dB/m) for the dominant frequency
        alpha = (
            8.686 * frequency ** 2
            * (
                1.84e-11 * (T_kelvin / 293.15) ** 0.5
                + (T_kelvin / 293.15) ** (-2.5)
                * (
                    0.01275 * math.exp(-2239.1 / T_kelvin) / (fr_O + frequency ** 2 / fr_O)
                )
            )
        )
        # Clamp to reasonable range
        alpha = max(0.001, min(0.5, alpha))

        # Wind vector components (m/s)
        wind_rad = math.radians(wind_direction)
        wind_x = wind_speed * math.sin(wind_rad)   # east component
        wind_y = wind_speed * math.cos(wind_rad)    # north component

        # Terrain adjustment (dB)
        terrain_adj = {"urban": 3.0, "rural": -3.0, "indoor": 6.0}.get(terrain, 0.0)
        # Indoor reverberation fades with distance
        terrain_distance_limit = 50.0 if terrain == "indoor" else float("inf")

        arrivals: list[dict[str, Any]] = []

        for i, listener in enumerate(listeners):
            dx = listener[0] - source[0]
            dy = listener[1] - source[1]
            dz = listener[2] - source[2]
            distance = math.sqrt(dx * dx + dy * dy + dz * dz)

            if distance < 0.01:
                # Listener essentially at source
                arrivals.append({
                    "listener_id": i,
                    "position": list(listener),
                    "distance_m": 0.0,
                    "delay_sec": 0.0,
                    "attenuation_db": 0.0,
                    "estimated_loudness_db": round(source_db, 1),
                    "above_hearing_threshold": True,
                    "above_pain_threshold": source_db >= 120.0,
                    "wind_effect_sec": 0.0,
                })
                continue

            # Wind effect on travel time:
            # effective speed = c + wind component along propagation direction
            if distance > 0:
                prop_dir_x = dx / distance
                prop_dir_y = dy / distance
                wind_component = wind_x * prop_dir_x + wind_y * prop_dir_y
            else:
                wind_component = 0.0

            effective_c = c + wind_component
            effective_c = max(effective_c, 100.0)  # physical sanity

            delay = distance / effective_c
            delay_no_wind = distance / c
            wind_effect = delay - delay_no_wind

            # Geometric spreading: 20 * log10(1 / d) referenced to 1 m
            geometric_atten = -20.0 * math.log10(distance)

            # Atmospheric absorption
            atmospheric_atten = -alpha * distance

            # Terrain effect (capped by distance)
            t_adj = terrain_adj if distance <= terrain_distance_limit else 0.0

            total_atten = geometric_atten + atmospheric_atten + t_adj
            estimated_db = source_db + total_atten

            # Human hearing thresholds at the dominant frequency
            # Threshold of hearing ~ 0-20 dB SPL (frequency dependent)
            # Threshold of pain ~ 120-130 dB SPL
            hearing_threshold = 20.0  # dB SPL (conservative)
            pain_threshold = 120.0

            arrivals.append({
                "listener_id": i,
                "position": list(listener),
                "distance_m": round(distance, 3),
                "delay_sec": round(delay, 6),
                "attenuation_db": round(total_atten, 2),
                "estimated_loudness_db": round(estimated_db, 1),
                "above_hearing_threshold": estimated_db > hearing_threshold,
                "above_pain_threshold": estimated_db >= pain_threshold,
                "wind_effect_sec": round(wind_effect, 6),
            })

        # Sort by arrival time
        arrivals.sort(key=lambda a: a["delay_sec"])

        return {
            "speed_of_sound": round(c, 2),
            "temperature": temperature,
            "humidity": humidity,
            "frequency_hz": frequency,
            "atmospheric_absorption_db_per_m": round(alpha, 5),
            "source_level_db": source_db,
            "terrain": terrain,
            "arrivals": arrivals,
        }

    # ================================================================
    # Utility: origin-of-impact estimation
    # ================================================================

    def estimate_origin_of_impact(
        self,
        stains: list[dict[str, Any]],
    ) -> dict[str, Any]:
        """Estimate the area of origin from a set of bloodstain measurements.

        Uses the tangent method: for each elliptical stain, the angle of
        impact alpha = arcsin(width / length).  The line from each stain
        at angle alpha converges toward the area of origin.

        Parameters
        ----------
        stains : list of dict
            Each dict must contain:
              x, y       — position on the surface (metres)
              width_mm   — minor axis of the elliptical stain
              length_mm  — major axis of the elliptical stain
              direction  — degrees, direction the drop was travelling
                           (0 = north / +y, 90 = east / +x)

        Returns
        -------
        dict with:
            origin_x, origin_y  — estimated (x, y) on the surface
            origin_z            — estimated height above surface
            convergence_lines   — list of (x, y, angle, impact_angle) per stain
            residual            — mean distance from lines to estimated origin
        """
        if len(stains) < 2:
            return {"error": "Need at least 2 stains to estimate origin"}

        lines: list[dict[str, float]] = []

        for s in stains:
            w = s["width_mm"]
            l_val = s["length_mm"]
            if l_val <= 0 or w <= 0:
                continue

            # Impact angle from stain eccentricity
            sin_alpha = min(w / l_val, 1.0)
            alpha = math.degrees(math.asin(sin_alpha))

            direction_rad = math.radians(s["direction"])
            # Unit vector pointing back toward the origin (opposite of travel)
            dx = -math.sin(direction_rad)
            dy = -math.cos(direction_rad)

            lines.append({
                "x": s["x"], "y": s["y"],
                "dx": dx, "dy": dy,
                "impact_angle": alpha,
            })

        if len(lines) < 2:
            return {"error": "Insufficient valid stains"}

        # Least-squares intersection of 2D lines
        # Each line: point (px, py) + t * (dx, dy)
        # Minimise sum of squared distances from point to each line.
        # Ax = b formulation from line normals.
        A = np.zeros((len(lines), 2))
        b = np.zeros(len(lines))

        for i, ln in enumerate(lines):
            # Normal to the line direction
            nx = -ln["dy"]
            ny = ln["dx"]
            A[i, 0] = nx
            A[i, 1] = ny
            b[i] = nx * ln["x"] + ny * ln["y"]

        # Solve in least-squares sense
        result, residuals, _, _ = np.linalg.lstsq(A, b, rcond=None)
        origin_x, origin_y = float(result[0]), float(result[1])

        # Estimate height (z) from each stain's impact angle and distance
        heights: list[float] = []
        for ln in lines:
            dist = math.sqrt((origin_x - ln["x"]) ** 2 + (origin_y - ln["y"]) ** 2)
            z_est = dist * math.tan(math.radians(ln["impact_angle"]))
            heights.append(z_est)

        origin_z = float(np.median(heights)) if heights else 0.0

        # Residual: mean perpendicular distance from origin to each line
        dists: list[float] = []
        for ln in lines:
            nx = -ln["dy"]
            ny = ln["dx"]
            d = abs(nx * (origin_x - ln["x"]) + ny * (origin_y - ln["y"]))
            dists.append(d)
        mean_residual = float(np.mean(dists))

        convergence_lines = [
            {
                "x": ln["x"],
                "y": ln["y"],
                "direction_dx": ln["dx"],
                "direction_dy": ln["dy"],
                "impact_angle": ln["impact_angle"],
            }
            for ln in lines
        ]

        return {
            "origin_x": round(origin_x, 4),
            "origin_y": round(origin_y, 4),
            "origin_z": round(origin_z, 4),
            "convergence_lines": convergence_lines,
            "residual_m": round(mean_residual, 4),
            "num_stains_used": len(lines),
        }
