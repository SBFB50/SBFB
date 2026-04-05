"""Tests for geo mapper (nexus.core.geo_mapper) with mocked HTTP.

No real HTTP calls -- everything is mocked.
"""

import pytest
from unittest.mock import AsyncMock, patch, MagicMock

from nexus.core.geo_mapper import GeoMapper, _guess_location_type


# =====================================================================
# _guess_location_type helper
# =====================================================================


class TestGuessLocationType:

    def test_crime_scene(self):
        ent = {"name": "scene du crime", "description": "lieu du meurtre"}
        assert _guess_location_type(ent) == "crime_scene"

    def test_home(self):
        ent = {"name": "domicile de Jean", "description": ""}
        assert _guess_location_type(ent) == "home"

    def test_work(self):
        ent = {"name": "bureau central", "description": "lieu de travail"}
        assert _guess_location_type(ent) == "work"

    def test_hospital(self):
        ent = {"name": "Hopital Saint-Louis", "description": ""}
        assert _guess_location_type(ent) == "hospital"

    def test_establishment(self):
        ent = {"name": "Bar Le Central", "description": "bar"}
        assert _guess_location_type(ent) == "establishment"

    def test_other(self):
        ent = {"name": "Pont Neuf", "description": "bridge"}
        assert _guess_location_type(ent) == "other"

    def test_handles_none_fields(self):
        ent = {"name": None, "description": None}
        assert _guess_location_type(ent) == "other"


# =====================================================================
# GeoMapper.geocode_address (mocked HTTP)
# =====================================================================


class TestGeocodeAddress:

    @pytest.mark.asyncio
    async def test_geocode_success(self):
        mock_db = AsyncMock()
        mapper = GeoMapper(mock_db)

        mock_response = MagicMock()
        mock_response.status_code = 200
        mock_response.raise_for_status = MagicMock()
        mock_response.json.return_value = [
            {"lat": "48.8566", "lon": "2.3522", "display_name": "Paris, France"}
        ]

        with patch("nexus.core.geo_mapper.httpx.AsyncClient") as MockClient:
            mock_client = AsyncMock()
            mock_client.get.return_value = mock_response
            mock_client.__aenter__ = AsyncMock(return_value=mock_client)
            mock_client.__aexit__ = AsyncMock(return_value=None)
            MockClient.return_value = mock_client

            result = await mapper.geocode_address("Paris")
            assert result is not None
            assert result["lat"] == 48.8566
            assert result["lon"] == 2.3522
            assert "Paris" in result["display_name"]

    @pytest.mark.asyncio
    async def test_geocode_no_result(self):
        mock_db = AsyncMock()
        mapper = GeoMapper(mock_db)

        mock_response = MagicMock()
        mock_response.status_code = 200
        mock_response.raise_for_status = MagicMock()
        mock_response.json.return_value = []

        with patch("nexus.core.geo_mapper.httpx.AsyncClient") as MockClient:
            mock_client = AsyncMock()
            mock_client.get.return_value = mock_response
            mock_client.__aenter__ = AsyncMock(return_value=mock_client)
            mock_client.__aexit__ = AsyncMock(return_value=None)
            MockClient.return_value = mock_client

            result = await mapper.geocode_address("nonexistent_place_xyz123")
            assert result is None

    @pytest.mark.asyncio
    async def test_geocode_http_error(self):
        import httpx

        mock_db = AsyncMock()
        mapper = GeoMapper(mock_db)

        with patch("nexus.core.geo_mapper.httpx.AsyncClient") as MockClient:
            mock_client = AsyncMock()
            mock_client.get.side_effect = httpx.ConnectError("Connection refused")
            mock_client.__aenter__ = AsyncMock(return_value=mock_client)
            mock_client.__aexit__ = AsyncMock(return_value=None)
            MockClient.return_value = mock_client

            result = await mapper.geocode_address("Paris")
            assert result is None


# =====================================================================
# GeoMapper.build_case_map_data (mocked DB)
# =====================================================================


class TestBuildCaseMapData:

    @pytest.mark.asyncio
    async def test_empty_case(self):
        mock_db = AsyncMock()
        mock_db.list_locations_by_case.return_value = []
        mapper = GeoMapper(mock_db)

        result = await mapper.build_case_map_data("case-1")
        assert result["locations"] == []
        assert result["entities_at_locations"] == []

    @pytest.mark.asyncio
    async def test_with_locations_and_entities(self):
        mock_db = AsyncMock()
        mock_db.list_locations_by_case.return_value = [
            {"id": "loc-1", "name": "Scene", "entity_id": "ent-1"},
        ]
        mock_db.get_entity.return_value = {
            "id": "ent-1",
            "name": "Crime Scene",
            "entity_type": "location",
        }
        mapper = GeoMapper(mock_db)

        result = await mapper.build_case_map_data("case-1")
        assert len(result["locations"]) == 1
        assert len(result["entities_at_locations"]) == 1
        assert result["entities_at_locations"][0]["entity_name"] == "Crime Scene"

    @pytest.mark.asyncio
    async def test_location_without_entity(self):
        mock_db = AsyncMock()
        mock_db.list_locations_by_case.return_value = [
            {"id": "loc-1", "name": "Bridge", "entity_id": None},
        ]
        mapper = GeoMapper(mock_db)

        result = await mapper.build_case_map_data("case-1")
        assert len(result["locations"]) == 1
        assert len(result["entities_at_locations"]) == 0


# =====================================================================
# GeoMapper.verify_travel_time (fully mocked)
# =====================================================================


class TestVerifyTravelTime:

    @pytest.mark.asyncio
    async def test_plausible_travel_time(self):
        mock_db = AsyncMock()
        mapper = GeoMapper(mock_db)

        # Mock calculate_route to return a known result
        with patch.object(mapper, "calculate_route", new_callable=AsyncMock) as mock_route:
            mock_route.return_value = {
                "distance_km": 50.0,
                "duration_min": 40.0,
                "geometry_geojson": {},
            }
            result = await mapper.verify_travel_time("A", "B", claimed_minutes=45.0)

        assert result is not None
        assert result["plausible"] is True
        assert result["actual_minutes"] == 40.0
        assert result["claimed_minutes"] == 45.0

    @pytest.mark.asyncio
    async def test_implausible_travel_time(self):
        mock_db = AsyncMock()
        mapper = GeoMapper(mock_db)

        with patch.object(mapper, "calculate_route", new_callable=AsyncMock) as mock_route:
            mock_route.return_value = {
                "distance_km": 500.0,
                "duration_min": 300.0,
                "geometry_geojson": {},
            }
            result = await mapper.verify_travel_time("A", "B", claimed_minutes=10.0)

        assert result is not None
        assert result["plausible"] is False

    @pytest.mark.asyncio
    async def test_route_not_found(self):
        mock_db = AsyncMock()
        mapper = GeoMapper(mock_db)

        with patch.object(mapper, "calculate_route", new_callable=AsyncMock) as mock_route:
            mock_route.return_value = None
            result = await mapper.verify_travel_time("A", "B", claimed_minutes=30.0)

        assert result is None
