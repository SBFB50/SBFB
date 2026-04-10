"""nexus-app-gov — political contradiction detection as a nexus-grid app."""

from nexus_app_gov.app import GovApp
from nexus_app_gov.prompts import POLITICAL_CONTRADICTION_PROMPT

__version__ = "0.1.0"

__all__ = ["GovApp", "POLITICAL_CONTRADICTION_PROMPT", "__version__"]
