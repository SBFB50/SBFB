"""
NEXUS -- HTTP client for the FastAPI backend.

Synchronous wrapper around ``requests`` designed for Streamlit.
All methods return parsed JSON (dict / list) or raise on error.
A module-level singleton ``api`` is provided for convenience.
"""

from __future__ import annotations

from typing import Any, Dict, List, Optional

import requests
import streamlit as st


class NexusAPIClient:
    """Thin synchronous client that maps 1-to-1 to NEXUS REST endpoints."""

    def __init__(self, base_url: str = "http://localhost:8000") -> None:
        self.base_url = base_url.rstrip("/")
        self._session = requests.Session()

    # ------------------------------------------------------------------
    # Internal helpers
    # ------------------------------------------------------------------

    def _url(self, path: str) -> str:
        return f"{self.base_url}{path}"

    def _request(
        self,
        method: str,
        path: str,
        *,
        json: Any = None,
        params: Optional[Dict[str, Any]] = None,
        files: Any = None,
        data: Any = None,
        raw: bool = False,
    ) -> Any:
        """Execute an HTTP request and return the parsed JSON response.

        If *raw* is True the raw ``Response`` object is returned instead
        (useful for binary downloads).
        """
        # Strip None values from params
        if params:
            params = {k: v for k, v in params.items() if v is not None}

        try:
            resp = self._session.request(
                method,
                self._url(path),
                json=json,
                params=params,
                files=files,
                data=data,
                timeout=120,
            )
        except requests.ConnectionError:
            st.error(
                "Impossible de joindre l'API NEXUS "
                f"({self.base_url}). Le backend est-il demarre?"
            )
            return None

        if raw:
            resp.raise_for_status()
            return resp

        # 204 No Content
        if resp.status_code == 204:
            return None

        if resp.status_code >= 400:
            detail = ""
            try:
                detail = resp.json().get("detail", resp.text)
            except Exception:
                detail = resp.text
            st.error(f"Erreur API ({resp.status_code}): {detail}")
            return None

        return resp.json()

    # ================================================================
    # Cases
    # ================================================================

    def create_case(self, data: Dict[str, Any]) -> Optional[Dict]:
        return self._request("POST", "/api/cases", json=data)

    def list_cases(self, status: Optional[str] = None) -> List[Dict]:
        return self._request("GET", "/api/cases", params={"status": status}) or []

    def get_case(self, case_id: str) -> Optional[Dict]:
        return self._request("GET", f"/api/cases/{case_id}")

    def update_case(self, case_id: str, data: Dict[str, Any]) -> Optional[Dict]:
        return self._request("PUT", f"/api/cases/{case_id}", json=data)

    def delete_case(self, case_id: str) -> None:
        self._request("DELETE", f"/api/cases/{case_id}")

    def get_case_stats(self, case_id: str) -> Optional[Dict]:
        return self._request("GET", f"/api/cases/{case_id}/stats")

    # ================================================================
    # Evidence
    # ================================================================

    def upload_evidence(
        self,
        case_id: str,
        file,
        title: str,
        source: Optional[str] = None,
    ) -> Optional[Dict]:
        """Upload a file as evidence (multipart/form-data)."""
        files = {"file": (file.name, file, file.type or "application/octet-stream")}
        form_data: Dict[str, Any] = {"title": title}
        if source:
            form_data["source"] = source
        return self._request(
            "POST",
            f"/api/cases/{case_id}/evidence",
            files=files,
            data=form_data,
        )

    def submit_text_evidence(
        self,
        case_id: str,
        title: str,
        text: str,
        source: Optional[str] = None,
    ) -> Optional[Dict]:
        return self._request(
            "POST",
            f"/api/cases/{case_id}/evidence/text",
            json={"title": title, "text": text, "source": source},
        )

    def list_evidence(
        self,
        case_id: str,
        status: Optional[str] = None,
        evidence_type: Optional[str] = None,
    ) -> List[Dict]:
        return (
            self._request(
                "GET",
                f"/api/cases/{case_id}/evidence",
                params={"status": status, "evidence_type": evidence_type},
            )
            or []
        )

    def get_evidence(self, evidence_id: str) -> Optional[Dict]:
        return self._request("GET", f"/api/evidence/{evidence_id}")

    def update_evidence(self, evidence_id: str, data: Dict[str, Any]) -> Optional[Dict]:
        return self._request("PUT", f"/api/evidence/{evidence_id}", json=data)

    def delete_evidence(self, evidence_id: str) -> None:
        self._request("DELETE", f"/api/evidence/{evidence_id}")

    # ================================================================
    # Entities
    # ================================================================

    def list_entities(
        self,
        case_id: str,
        entity_type: Optional[str] = None,
    ) -> List[Dict]:
        return (
            self._request(
                "GET",
                f"/api/cases/{case_id}/entities",
                params={"entity_type": entity_type},
            )
            or []
        )

    def get_entity(self, entity_id: str) -> Optional[Dict]:
        return self._request("GET", f"/api/entities/{entity_id}")

    def get_entity_mentions(self, entity_id: str) -> List[Dict]:
        return self._request("GET", f"/api/entities/{entity_id}/mentions") or []

    # ================================================================
    # Analysis
    # ================================================================

    def trigger_analysis(self, case_id: str) -> Optional[Dict]:
        return self._request("POST", f"/api/cases/{case_id}/analyze")

    def get_analysis_run(self, run_id: str) -> Optional[Dict]:
        return self._request("GET", f"/api/analysis/{run_id}")

    def list_analysis_runs(self, case_id: str) -> List[Dict]:
        return self._request("GET", f"/api/cases/{case_id}/analysis-runs") or []

    # ================================================================
    # Graph (Neo4j)
    # ================================================================

    def get_graph(self, case_id: str) -> Optional[Dict]:
        return self._request("GET", f"/api/cases/{case_id}/graph")

    def get_neighbors(
        self, case_id: str, node_id: str, depth: int = 1
    ) -> Optional[Dict]:
        return self._request(
            "GET",
            f"/api/cases/{case_id}/graph/neighbors/{node_id}",
            params={"depth": depth},
        )

    def get_shortest_path(
        self, case_id: str, from_id: str, to_id: str
    ) -> Optional[Dict]:
        return self._request(
            "GET", f"/api/cases/{case_id}/graph/path/{from_id}/{to_id}"
        )

    def get_clusters(self, case_id: str) -> Optional[Dict]:
        return self._request("GET", f"/api/cases/{case_id}/graph/clusters")

    def get_graph_stats(self, case_id: str) -> Optional[Dict]:
        return self._request("GET", f"/api/cases/{case_id}/graph/stats")

    # ================================================================
    # Search (ChromaDB)
    # ================================================================

    def semantic_search(
        self,
        case_id: str,
        query: str,
        n_results: int = 10,
        collection: str = "evidence",
    ) -> Optional[Dict]:
        return self._request(
            "POST",
            f"/api/cases/{case_id}/search",
            json={
                "query": query,
                "n_results": n_results,
                "collection": collection,
            },
        )

    def find_similar(
        self, case_id: str, evidence_id: str, n_results: int = 5
    ) -> Optional[Dict]:
        return self._request(
            "GET",
            f"/api/cases/{case_id}/similar/{evidence_id}",
            params={"n_results": n_results},
        )

    def find_duplicates(
        self, case_id: str, threshold: float = 0.92
    ) -> Optional[Dict]:
        return self._request(
            "GET",
            f"/api/cases/{case_id}/duplicates",
            params={"threshold": threshold},
        )

    # ================================================================
    # Monitoring
    # ================================================================

    def create_monitoring_job(
        self, case_id: str, data: Dict[str, Any]
    ) -> Optional[Dict]:
        return self._request(
            "POST", f"/api/cases/{case_id}/monitoring", json=data
        )

    def list_monitoring_jobs(self, case_id: str) -> List[Dict]:
        return (
            self._request("GET", f"/api/cases/{case_id}/monitoring") or []
        )

    def trigger_monitoring_job(self, job_id: str) -> Optional[Dict]:
        return self._request("POST", f"/api/monitoring/{job_id}/run")

    def list_monitoring_results(self, case_id: str) -> List[Dict]:
        return (
            self._request("GET", f"/api/cases/{case_id}/monitoring/results")
            or []
        )

    def ingest_monitoring_result(self, result_id: str) -> Optional[Dict]:
        return self._request(
            "POST", f"/api/monitoring/results/{result_id}/ingest"
        )

    # ================================================================
    # Alerts
    # ================================================================

    def list_alerts(
        self,
        case_id: str,
        severity: Optional[str] = None,
        unread_only: bool = False,
    ) -> List[Dict]:
        return (
            self._request(
                "GET",
                f"/api/cases/{case_id}/alerts",
                params={"severity": severity, "unread_only": unread_only},
            )
            or []
        )

    def mark_alert_read(self, alert_id: str) -> Optional[Dict]:
        return self._request("PUT", f"/api/alerts/{alert_id}/read")

    def get_unread_count(self, case_id: Optional[str] = None) -> int:
        result = self._request(
            "GET", "/api/alerts/unread-count", params={"case_id": case_id}
        )
        if result is None:
            return 0
        return result.get("unread_count", 0)

    # ================================================================
    # Hypotheses
    # ================================================================

    def create_hypothesis(
        self, case_id: str, data: Dict[str, Any]
    ) -> Optional[Dict]:
        return self._request(
            "POST", f"/api/cases/{case_id}/hypotheses", json=data
        )

    def list_hypotheses(
        self, case_id: str, status: Optional[str] = None
    ) -> List[Dict]:
        return (
            self._request(
                "GET",
                f"/api/cases/{case_id}/hypotheses",
                params={"status": status},
            )
            or []
        )

    def get_hypothesis(self, hyp_id: str) -> Optional[Dict]:
        return self._request("GET", f"/api/hypotheses/{hyp_id}")

    def update_hypothesis(
        self, hyp_id: str, data: Dict[str, Any]
    ) -> Optional[Dict]:
        return self._request("PUT", f"/api/hypotheses/{hyp_id}", json=data)

    def evaluate_hypothesis(self, hyp_id: str) -> Optional[Dict]:
        return self._request("POST", f"/api/hypotheses/{hyp_id}/evaluate")

    def get_hypothesis_snapshots(self, hyp_id: str) -> List[Dict]:
        return (
            self._request("GET", f"/api/hypotheses/{hyp_id}/snapshots") or []
        )

    def get_hypothesis_evolution(self, hyp_id: str) -> List[Dict]:
        return (
            self._request("GET", f"/api/hypotheses/{hyp_id}/evolution") or []
        )

    def generate_hypotheses(self, case_id: str) -> Optional[Dict]:
        return self._request(
            "POST", f"/api/cases/{case_id}/hypotheses/generate"
        )

    def evaluate_all_hypotheses(self, case_id: str) -> Optional[Dict]:
        return self._request("POST", f"/api/cases/{case_id}/evaluate-all")

    def get_contradictions(self, case_id: str) -> List[Dict]:
        return (
            self._request("GET", f"/api/cases/{case_id}/contradictions") or []
        )

    # ================================================================
    # Reports
    # ================================================================

    def generate_report(
        self, case_id: str, report_type: str = "full"
    ) -> Optional[Dict]:
        return self._request(
            "POST",
            f"/api/cases/{case_id}/reports/generate",
            json={"report_type": report_type},
        )

    def get_report(self, report_id: str) -> Optional[Dict]:
        return self._request("GET", f"/api/reports/{report_id}")

    def download_report(self, report_id: str) -> Optional[bytes]:
        resp = self._request(
            "GET", f"/api/reports/{report_id}/download", raw=True
        )
        if resp is not None:
            return resp.content
        return None

    def list_reports(self, case_id: str) -> List[Dict]:
        return self._request("GET", f"/api/cases/{case_id}/reports") or []

    # ================================================================
    # Timeline
    # ================================================================

    def get_timeline(self, case_id: str) -> List[Dict]:
        return (
            self._request("GET", f"/api/cases/{case_id}/timeline") or []
        )

    # ================================================================
    # Geo / Map
    # ================================================================

    def geocode_case(self, case_id: str) -> Optional[Dict]:
        """Geocode all location entities for a case."""
        return self._request("POST", f"/api/cases/{case_id}/geocode")

    def get_case_map(self, case_id: str) -> Optional[Dict]:
        """Retrieve full map data (locations, entities) for a case."""
        return self._request("GET", f"/api/cases/{case_id}/map")

    def calculate_route(
        self, case_id: str, origin: str, destination: str
    ) -> Optional[Dict]:
        """Calculate a driving route between two addresses."""
        return self._request(
            "POST",
            f"/api/cases/{case_id}/route",
            json={"origin": origin, "destination": destination},
        )

    def verify_travel(
        self,
        case_id: str,
        origin: str,
        destination: str,
        claimed_minutes: float,
    ) -> Optional[Dict]:
        """Verify whether a claimed travel time is plausible."""
        return self._request(
            "POST",
            f"/api/cases/{case_id}/verify-travel",
            json={
                "origin": origin,
                "destination": destination,
                "claimed_minutes": claimed_minutes,
            },
        )

    # ================================================================
    # OSINT Recon
    # ================================================================

    def recon_email(self, email: str) -> Optional[Dict]:
        return self._request("POST", f"/api/recon/email/{email}")

    def recon_username(self, username: str) -> Optional[Dict]:
        return self._request("POST", f"/api/recon/username/{username}")

    def recon_domain(self, domain: str) -> Optional[Dict]:
        return self._request("POST", f"/api/recon/domain/{domain}")

    def get_case_recon(self, case_id: str) -> List[Dict]:
        return self._request("GET", f"/api/cases/{case_id}/recon") or []

    def recon_auto(self, case_id: str) -> Optional[Dict]:
        return self._request("POST", f"/api/cases/{case_id}/recon/auto")

    # ================================================================
    # Image Search (Visual Embeddings)
    # ================================================================

    def search_images_by_text(
        self,
        case_id: str,
        query: str,
        n_results: int = 5,
    ) -> Optional[List[Dict]]:
        """Search images by natural language query using CLIP."""
        return self._request(
            "POST",
            f"/api/cases/{case_id}/images/search-by-text",
            json={"query": query, "n_results": n_results},
        )

    def search_images_by_image(
        self,
        case_id: str,
        evidence_id: str,
        n_results: int = 5,
    ) -> Optional[List[Dict]]:
        """Search for visually similar images using DINOv2."""
        return self._request(
            "POST",
            f"/api/cases/{case_id}/images/search-by-image",
            json={"evidence_id": evidence_id, "n_results": n_results},
        )

    def get_similar_images(
        self,
        case_id: str,
        evidence_id: str,
        n_results: int = 5,
    ) -> List[Dict]:
        """Find images similar to an already-indexed evidence image."""
        return (
            self._request(
                "GET",
                f"/api/cases/{case_id}/images/similar/{evidence_id}",
                params={"n_results": n_results},
            )
            or []
        )

    def index_case_images(self, case_id: str) -> Optional[Dict]:
        """Index all image evidence for a case into visual search."""
        return self._request(
            "POST", f"/api/cases/{case_id}/images/index"
        )

    # ================================================================
    # Vision (VLM Analysis)
    # ================================================================

    def analyze_evidence_image(self, evidence_id: str) -> Optional[Dict]:
        """Run the full visual analysis pipeline on an evidence image."""
        return self._request("POST", f"/api/evidence/{evidence_id}/analyze-image")

    def analyze_all_case_images(self, case_id: str) -> Optional[Dict]:
        """Analyze all image evidence in a case."""
        return self._request("POST", f"/api/cases/{case_id}/analyze-images")

    def describe_image_direct(self, file) -> Optional[Dict]:
        """Upload an image for direct description (no evidence creation)."""
        files = {"file": (file.name, file, file.type or "image/jpeg")}
        return self._request("POST", "/api/vision/describe", files=files)

    def compare_evidence_images(
        self, evidence_id_1: str, evidence_id_2: str
    ) -> Optional[Dict]:
        """Compare two evidence images."""
        return self._request(
            "POST",
            "/api/vision/compare",
            data={
                "evidence_id_1": evidence_id_1,
                "evidence_id_2": evidence_id_2,
            },
        )

    def list_visual_entities(self, case_id: str) -> List[Dict]:
        """List entities extracted from images for a case."""
        return self._request("GET", f"/api/cases/{case_id}/visual-entities") or []

    # ================================================================
    # Forensics
    # ================================================================

    def forensic_bpa_classify(self, file) -> Optional[Dict]:
        """Classify a blood pattern from an uploaded image."""
        files = {"file": (file.name, file, file.type or "image/jpeg")}
        return self._request("POST", "/api/forensics/bpa/classify", files=files)

    def forensic_bpa_analyze(
        self,
        file,
        measurements: Optional[str] = None,
        case_context: str = "",
    ) -> Optional[Dict]:
        """Full BPA analysis with optional measurements."""
        files = {"file": (file.name, file, file.type or "image/jpeg")}
        data: Dict[str, Any] = {"case_context": case_context}
        if measurements:
            data["measurements"] = measurements
        return self._request(
            "POST", "/api/forensics/bpa/analyze", files=files, data=data
        )

    def forensic_bpa_calculate_angle(
        self, width: float, length: float
    ) -> Optional[Dict]:
        """Calculate impact angle from stain dimensions."""
        return self._request(
            "POST",
            "/api/forensics/bpa/calculate-angle",
            json={"width": width, "length": length},
        )

    def forensic_bpa_convergence(
        self, stains: List[Dict]
    ) -> Optional[Dict]:
        """Calculate area of convergence from stain measurements."""
        return self._request(
            "POST",
            "/api/forensics/bpa/convergence",
            json={"stains": stains},
        )

    def forensic_audio_transcribe(self, file) -> Optional[Dict]:
        """Transcribe an audio file."""
        files = {"file": (file.name, file, file.type or "audio/wav")}
        return self._request(
            "POST", "/api/forensics/audio/transcribe", files=files
        )

    def forensic_audio_analyze(self, file) -> Optional[Dict]:
        """Full forensic audio analysis."""
        files = {"file": (file.name, file, file.type or "audio/wav")}
        return self._request(
            "POST", "/api/forensics/audio/analyze", files=files
        )

    def forensic_audio_events(self, file) -> Optional[Dict]:
        """Detect events in an audio file."""
        files = {"file": (file.name, file, file.type or "audio/wav")}
        return self._request(
            "POST", "/api/forensics/audio/events", files=files
        )

    def forensic_trace_analyze(
        self, file, trace_type: str = "auto"
    ) -> Optional[Dict]:
        """Analyze a physical trace from a photo."""
        files = {"file": (file.name, file, file.type or "image/jpeg")}
        data = {"trace_type": trace_type}
        return self._request(
            "POST", "/api/forensics/trace/analyze", files=files, data=data
        )

    def forensic_trace_compare(self, file_1, file_2) -> Optional[Dict]:
        """Compare two trace images."""
        files = {
            "file_1": (file_1.name, file_1, file_1.type or "image/jpeg"),
            "file_2": (file_2.name, file_2, file_2.type or "image/jpeg"),
        }
        return self._request(
            "POST", "/api/forensics/trace/compare", files=files
        )

    def forensic_auto_analyze(self, case_id: str) -> Optional[Dict]:
        """Run automatic forensic analysis on all case evidence."""
        return self._request(
            "POST", f"/api/forensics/cases/{case_id}/auto"
        )

    # ================================================================
    # Physics Simulations
    # ================================================================

    def sim_blood_drop(
        self,
        velocity: float,
        angle: float,
        height: float,
        surface_angle: float = 0.0,
        blood_properties: Optional[Dict[str, float]] = None,
    ) -> Optional[Dict]:
        """Simulate a single blood drop trajectory and impact."""
        payload: Dict[str, Any] = {
            "velocity": velocity,
            "angle": angle,
            "height": height,
            "surface_angle": surface_angle,
        }
        if blood_properties:
            payload["blood_properties"] = blood_properties
        return self._request(
            "POST", "/api/forensics/sim/blood-drop", json=payload
        )

    def sim_cast_off(
        self,
        swing_radius: float,
        swing_speed: float,
        num_drops: int = 20,
        blood_on_weapon_length: float = 0.3,
        swing_plane_height: float = 1.5,
        swing_start_angle: float = -30.0,
        swing_end_angle: float = 150.0,
        blood_properties: Optional[Dict[str, float]] = None,
    ) -> Optional[Dict]:
        """Simulate a cast-off blood pattern from a swinging weapon."""
        payload: Dict[str, Any] = {
            "swing_radius": swing_radius,
            "swing_speed": swing_speed,
            "num_drops": num_drops,
            "blood_on_weapon_length": blood_on_weapon_length,
            "swing_plane_height": swing_plane_height,
            "swing_start_angle": swing_start_angle,
            "swing_end_angle": swing_end_angle,
        }
        if blood_properties:
            payload["blood_properties"] = blood_properties
        return self._request(
            "POST", "/api/forensics/sim/cast-off", json=payload
        )

    def sim_sound(
        self,
        source: List[float],
        listeners: List[List[float]],
        source_db: float = 160.0,
        frequency: float = 2000.0,
        temperature: float = 20.0,
        humidity: float = 50.0,
        wind_speed: float = 0.0,
        wind_direction: float = 0.0,
        terrain: str = "urban",
    ) -> Optional[Dict]:
        """Simulate sound propagation from source to listeners."""
        return self._request(
            "POST",
            "/api/forensics/sim/sound",
            json={
                "source": source,
                "listeners": listeners,
                "source_db": source_db,
                "frequency": frequency,
                "temperature": temperature,
                "humidity": humidity,
                "wind_speed": wind_speed,
                "wind_direction": wind_direction,
                "terrain": terrain,
            },
        )

    def sim_origin(
        self, stains: List[Dict[str, Any]]
    ) -> Optional[Dict]:
        """Estimate origin of impact from stain measurements."""
        return self._request(
            "POST",
            "/api/forensics/sim/origin",
            json={"stains": stains},
        )

    def list_sim_datasets(self) -> Optional[Dict]:
        """List physics simulation datasets from The Well."""
        return self._request("GET", "/api/forensics/sim/datasets")

    # ================================================================
    # Autonomous Investigation
    # ================================================================

    def list_investigations(self) -> Optional[Dict]:
        """Get status of all active autonomous investigations."""
        return self._request("GET", "/api/investigations")

    def start_investigation(self, case_id: str) -> Optional[Dict]:
        """Start autonomous investigation for a case."""
        return self._request(
            "POST", f"/api/cases/{case_id}/investigation/start"
        )

    def stop_investigation(self, case_id: str) -> Optional[Dict]:
        """Stop autonomous investigation for a case."""
        return self._request(
            "POST", f"/api/cases/{case_id}/investigation/stop"
        )

    def get_investigation_status(self, case_id: str) -> Optional[Dict]:
        """Get detailed status of a case investigation."""
        return self._request(
            "GET", f"/api/cases/{case_id}/investigation/status"
        )

    def get_investigation_log(
        self, case_id: str, limit: int = 50
    ) -> List[Dict]:
        """Get the autonomous action log for a case."""
        return (
            self._request(
                "GET",
                f"/api/cases/{case_id}/investigation/log",
                params={"limit": limit},
            )
            or []
        )

    # ================================================================
    # Audit Trail
    # ================================================================

    def list_audit_log(
        self,
        case_id: str,
        action: Optional[str] = None,
        actor: Optional[str] = None,
        limit: int = 100,
        offset: int = 0,
    ) -> List[Dict]:
        return (
            self._request(
                "GET",
                f"/api/cases/{case_id}/audit",
                params={
                    "action": action,
                    "actor": actor,
                    "limit": limit,
                    "offset": offset,
                },
            )
            or []
        )

    def get_audit_summary(self, case_id: str) -> Optional[Dict]:
        return self._request("GET", f"/api/cases/{case_id}/audit/summary")

    def get_audit_timeline(self, case_id: str) -> List[Dict]:
        return (
            self._request("GET", f"/api/cases/{case_id}/audit/timeline")
            or []
        )

    def get_audit_entry(self, audit_id: str) -> Optional[Dict]:
        return self._request("GET", f"/api/audit/{audit_id}")

    # ================================================================
    # Health
    # ================================================================

    def check_health(self) -> Optional[Dict]:
        return self._request("GET", "/api/health")


# ------------------------------------------------------------------
# Module-level singleton
# ------------------------------------------------------------------

api = NexusAPIClient()
