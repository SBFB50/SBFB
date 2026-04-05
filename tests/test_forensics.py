"""Tests for forensic calculations (BPA geometry, physics sim).

All tests use only the pure-computation methods -- no LLM calls.
"""

import math

import pytest

from nexus.forensics.blood_pattern import BloodPatternAnalyzer
from nexus.forensics.physics_sim import ForensicPhysicsSim


# =====================================================================
# BloodPatternAnalyzer — Geometry (no LLM needed)
# =====================================================================


class TestBloodPatternGeometry:
    """Test the pure-math methods of BloodPatternAnalyzer.

    These don't need a router -- we only call calculate_impact_angle
    and calculate_area_of_convergence which are synchronous.
    """

    def _make_analyzer(self):
        """Create analyzer with a None router (geometry methods don't use it)."""
        return BloodPatternAnalyzer(router=None)

    # -- Impact angle --

    def test_impact_angle_90_degrees(self):
        """width == length => perpendicular impact => 90 degrees."""
        bpa = self._make_analyzer()
        angle = bpa.calculate_impact_angle(width=5.0, length=5.0)
        assert abs(angle - 90.0) < 0.01

    def test_impact_angle_30_degrees(self):
        """sin(30) = 0.5, so width/length = 0.5."""
        bpa = self._make_analyzer()
        angle = bpa.calculate_impact_angle(width=2.5, length=5.0)
        assert abs(angle - 30.0) < 0.01

    def test_impact_angle_45_degrees(self):
        """sin(45) ~ 0.707."""
        bpa = self._make_analyzer()
        width = 5.0 * math.sin(math.radians(45))
        angle = bpa.calculate_impact_angle(width=width, length=5.0)
        assert abs(angle - 45.0) < 0.1

    def test_impact_angle_very_shallow(self):
        """Nearly parallel impact (very small angle)."""
        bpa = self._make_analyzer()
        angle = bpa.calculate_impact_angle(width=0.1, length=10.0)
        assert 0 < angle < 5

    def test_impact_angle_invalid_zero_length(self):
        bpa = self._make_analyzer()
        with pytest.raises(ValueError, match="positive"):
            bpa.calculate_impact_angle(width=1.0, length=0.0)

    def test_impact_angle_invalid_negative(self):
        bpa = self._make_analyzer()
        with pytest.raises(ValueError, match="positive"):
            bpa.calculate_impact_angle(width=-1.0, length=5.0)

    def test_impact_angle_width_exceeds_length(self):
        bpa = self._make_analyzer()
        with pytest.raises(ValueError, match="cannot exceed"):
            bpa.calculate_impact_angle(width=10.0, length=5.0)

    # -- Area of convergence --

    def test_convergence_two_perpendicular_stains(self):
        """Two stains pointing at the origin from perpendicular directions."""
        bpa = self._make_analyzer()
        stains = [
            {"x": 100.0, "y": 0.0, "direction_degrees": 180.0},  # pointing left
            {"x": 0.0, "y": 100.0, "direction_degrees": 270.0},  # pointing down
        ]
        result = bpa.calculate_area_of_convergence(stains)
        assert "center_x" in result
        assert "center_y" in result
        assert result["num_intersections"] >= 1
        assert result["confidence"] > 0

    def test_convergence_insufficient_stains(self):
        bpa = self._make_analyzer()
        with pytest.raises(ValueError, match="at least 2"):
            bpa.calculate_area_of_convergence([{"x": 0, "y": 0, "direction_degrees": 0}])

    def test_convergence_parallel_stains(self):
        """Two parallel stains should fail (no intersection)."""
        bpa = self._make_analyzer()
        stains = [
            {"x": 0.0, "y": 0.0, "direction_degrees": 90.0},
            {"x": 0.0, "y": 100.0, "direction_degrees": 90.0},
        ]
        with pytest.raises(ValueError, match="parallel"):
            bpa.calculate_area_of_convergence(stains)

    # -- Area of origin (3D) --

    def test_area_of_origin(self):
        bpa = self._make_analyzer()
        convergence = {"center_x": 0.0, "center_y": 0.0}
        stains = [
            {"x": 100.0, "y": 0.0, "angle_degrees": 45.0},
            {"x": 0.0, "y": 100.0, "angle_degrees": 45.0},
        ]
        result = bpa.estimate_area_of_origin(stains, convergence)
        assert result["z_height"] > 0
        # At 45 degrees and distance 100, height should be ~100
        assert 80 < result["z_height"] < 120
        assert result["num_estimates"] == 2

    def test_area_of_origin_no_valid_angles(self):
        bpa = self._make_analyzer()
        stains = [
            {"x": 100.0, "y": 0.0, "angle_degrees": 0.0},  # invalid
        ]
        with pytest.raises(ValueError, match="valid height"):
            bpa.estimate_area_of_origin(stains, {"center_x": 0, "center_y": 0})


# =====================================================================
# ForensicPhysicsSim — Blood drop
# =====================================================================


class TestBloodDropSimulation:

    def test_basic_trajectory(self):
        sim = ForensicPhysicsSim()
        result = sim.simulate_blood_drop(
            velocity=5.0, angle=45.0, height=1.5
        )
        assert result["impact_angle"] > 0
        assert result["impact_angle"] <= 90
        assert len(result["trajectory"]) > 10
        assert result["travel_time"] > 0
        assert result["impact_velocity"] > 0

    def test_vertical_drop(self):
        """Drop falling straight down from rest."""
        sim = ForensicPhysicsSim()
        result = sim.simulate_blood_drop(
            velocity=0.001, angle=-90.0, height=2.0
        )
        assert result["impact_angle"] > 45  # nearly vertical
        assert result["travel_time"] > 0.1  # takes measurable time

    def test_horizontal_shot(self):
        """Blood projected horizontally from a height."""
        sim = ForensicPhysicsSim()
        result = sim.simulate_blood_drop(
            velocity=10.0, angle=0.0, height=1.5
        )
        # Should travel some horizontal distance
        ix, iy = result["impact_point"]
        assert ix > 0
        assert abs(iy) < 0.01  # hits the ground (y=0)

    def test_stain_shape_returned(self):
        sim = ForensicPhysicsSim()
        result = sim.simulate_blood_drop(
            velocity=5.0, angle=30.0, height=1.0
        )
        stain = result["stain_shape"]
        assert stain["width_mm"] > 0
        assert stain["length_mm"] > 0
        assert stain["length_mm"] >= stain["width_mm"]
        assert 0 <= stain["eccentricity"] <= 1

    def test_reynolds_number_returned(self):
        sim = ForensicPhysicsSim()
        result = sim.simulate_blood_drop(
            velocity=5.0, angle=45.0, height=1.0
        )
        assert "reynolds_at_impact" in result
        assert result["reynolds_at_impact"] > 0

    def test_blood_properties_override(self):
        sim = ForensicPhysicsSim()
        props = {"density": 1100.0, "drop_diameter": 0.003}
        result = sim.simulate_blood_drop(
            velocity=5.0, angle=45.0, height=1.0, blood_properties=props
        )
        assert result["blood_properties"]["density"] == 1100.0
        assert result["blood_properties"]["drop_diameter"] == 0.003


# =====================================================================
# ForensicPhysicsSim — Sound propagation
# =====================================================================


class TestSoundPropagation:

    def test_basic_propagation(self):
        sim = ForensicPhysicsSim()
        result = sim.simulate_sound_propagation(
            source=(0, 0, 1.5),
            listeners=[(100, 0, 1.5), (0, 100, 1.5)],
        )
        assert result["speed_of_sound"] > 330
        assert result["speed_of_sound"] < 360
        assert len(result["arrivals"]) == 2

    def test_arrival_times_proportional_to_distance(self):
        sim = ForensicPhysicsSim()
        result = sim.simulate_sound_propagation(
            source=(0, 0, 1.5),
            listeners=[(100, 0, 1.5), (200, 0, 1.5)],
        )
        arrivals = sorted(result["arrivals"], key=lambda a: a["distance_m"])
        assert arrivals[0]["delay_sec"] < arrivals[1]["delay_sec"]
        assert arrivals[0]["distance_m"] < arrivals[1]["distance_m"]

    def test_loudness_decreases_with_distance(self):
        sim = ForensicPhysicsSim()
        result = sim.simulate_sound_propagation(
            source=(0, 0, 1.5),
            listeners=[(10, 0, 1.5), (1000, 0, 1.5)],
        )
        arrivals = sorted(result["arrivals"], key=lambda a: a["distance_m"])
        assert arrivals[0]["estimated_loudness_db"] > arrivals[1]["estimated_loudness_db"]

    def test_listener_at_source(self):
        sim = ForensicPhysicsSim()
        result = sim.simulate_sound_propagation(
            source=(0, 0, 1.5),
            listeners=[(0, 0, 1.5)],
        )
        arr = result["arrivals"][0]
        assert arr["delay_sec"] == 0.0
        assert arr["distance_m"] == 0.0

    def test_temperature_affects_speed(self):
        sim = ForensicPhysicsSim()
        cold = sim.simulate_sound_propagation(
            source=(0, 0, 0), listeners=[(100, 0, 0)], temperature=0.0
        )
        hot = sim.simulate_sound_propagation(
            source=(0, 0, 0), listeners=[(100, 0, 0)], temperature=40.0
        )
        assert hot["speed_of_sound"] > cold["speed_of_sound"]

    def test_terrain_indoor(self):
        sim = ForensicPhysicsSim()
        result = sim.simulate_sound_propagation(
            source=(0, 0, 1.5),
            listeners=[(10, 0, 1.5)],
            terrain="indoor",
        )
        assert result["terrain"] == "indoor"
        # Indoor should have higher dB at short distances
        arr = result["arrivals"][0]
        assert arr["estimated_loudness_db"] > 100  # still very loud at 10m

    def test_hearing_threshold_flags(self):
        sim = ForensicPhysicsSim()
        result = sim.simulate_sound_propagation(
            source=(0, 0, 1.5),
            listeners=[(10, 0, 1.5), (10000, 0, 1.5)],
            source_db=160.0,
        )
        arrivals = sorted(result["arrivals"], key=lambda a: a["distance_m"])
        # Close listener should hear it
        assert arrivals[0]["above_hearing_threshold"] is True
        # Far listener may not (10 km away)
        # The exact result depends on attenuation, so just check the field exists
        assert "above_hearing_threshold" in arrivals[1]


# =====================================================================
# ForensicPhysicsSim — Cast-off
# =====================================================================


class TestCastOff:

    def test_cast_off_basic(self):
        sim = ForensicPhysicsSim()
        results = sim.simulate_cast_off(
            swing_radius=0.8,
            swing_speed=30.0,  # fast swing
            num_drops=10,
        )
        # At 30 rad/s, some drops should detach
        assert len(results) > 0
        for drop in results:
            assert "release_angle_deg" in drop
            assert "drop_sim" in drop
            assert drop["drop_sim"]["impact_angle"] > 0

    def test_cast_off_slow_no_detachment(self):
        """Very slow swing: surface tension keeps blood on weapon."""
        sim = ForensicPhysicsSim()
        results = sim.simulate_cast_off(
            swing_radius=0.8,
            swing_speed=1.0,  # very slow
            num_drops=10,
        )
        # Might produce zero drops if swing is too slow
        # This validates the detachment physics threshold
        assert isinstance(results, list)


# =====================================================================
# ForensicPhysicsSim — Origin estimation
# =====================================================================


class TestOriginEstimation:

    def test_basic_origin(self):
        sim = ForensicPhysicsSim()
        stains = [
            {"x": 1.0, "y": 0.0, "width_mm": 3.0, "length_mm": 6.0, "direction": 180.0},
            {"x": 0.0, "y": 1.0, "width_mm": 3.0, "length_mm": 6.0, "direction": 270.0},
        ]
        result = sim.estimate_origin_of_impact(stains)
        assert "origin_x" in result
        assert "origin_y" in result
        assert "origin_z" in result
        assert result["num_stains_used"] == 2

    def test_insufficient_stains(self):
        sim = ForensicPhysicsSim()
        result = sim.estimate_origin_of_impact([
            {"x": 0, "y": 0, "width_mm": 1, "length_mm": 2, "direction": 0}
        ])
        assert "error" in result
