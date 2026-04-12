# SPDX-License-Identifier: AGPL-3.0-or-later
"""nexus-coordinator — project-scoped coordinator for nexus-grid.

Public entry points:

- :class:`nexus_coordinator.coordinator.Coordinator`
- :class:`nexus_coordinator.config.CoordinatorConfig`
- :func:`nexus_coordinator.paths.project_dir`
- :func:`nexus_coordinator.keystore.load_or_generate_keypair`
"""

from nexus_coordinator.config import CoordinatorConfig
from nexus_coordinator.coordinator import Coordinator
from nexus_coordinator.paths import project_dir

__version__ = "0.1.0"

__all__ = ["Coordinator", "CoordinatorConfig", "project_dir", "__version__"]
