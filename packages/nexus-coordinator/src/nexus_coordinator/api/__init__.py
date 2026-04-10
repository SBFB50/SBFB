"""FastAPI control plane for the coordinator.

Phase A exposes :data:`/health` and :data:`/project` only. Phase
B/C/D attach additional routers under the same app factory.
"""

from nexus_coordinator.api.app import create_app

__all__ = ["create_app"]
