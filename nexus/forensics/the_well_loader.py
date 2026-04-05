"""
NEXUS -- PolymathicAI/the_well dataset loader.

Loads relevant physics simulation datasets from The Well for:
- Fluid dynamics reference data (applicable to Blood Pattern Analysis)
- Acoustic propagation / scattering data
- Training data for physics-informed models

The Well is a 15 TB collection of 16 physics simulation datasets stored
as HDF5 files on the HuggingFace Hub (polymathic-ai/*).

Requires: pip install the-well  (optional dependency)
Datasets are streamed from HuggingFace Hub on demand.

Reference:
  https://polymathic-ai.org/the_well/
  https://pypi.org/project/the-well/
"""

from __future__ import annotations

from typing import Any, Optional

from loguru import logger


# -----------------------------------------------------------------------
# Catalogue of forensics-relevant datasets from The Well
# -----------------------------------------------------------------------

_FORENSIC_DATASETS: list[dict[str, str]] = [
    {
        "name": "acoustic_scattering_inclusions",
        "type": "acoustic",
        "description": (
            "Variable-coefficient acoustic equations: propagation of pressure "
            "waves through domains with random inclusions of different "
            "scattering properties.  Applicable to gunshot / sound propagation "
            "modelling in complex environments."
        ),
        "relevance": "Sound propagation through walls, obstacles, urban terrain",
        "size_gb": "~30",
        "hf_path": "polymathic-ai/acoustic_scattering_inclusions",
    },
    {
        "name": "acoustic_scattering_discontinuous",
        "type": "acoustic",
        "description": (
            "Acoustic wave propagation with sharp density discontinuities. "
            "Models how sound refracts and reflects at material boundaries."
        ),
        "relevance": "Sound reflection off walls, floors, vehicles",
        "size_gb": "~25",
        "hf_path": "polymathic-ai/acoustic_scattering_discontinuous",
    },
    {
        "name": "shear_flow",
        "type": "fluid",
        "description": (
            "2D periodic shear flow governed by incompressible Navier-Stokes "
            "equations.  Captures vortex dynamics and mixing in viscous fluids."
        ),
        "relevance": "Blood flow patterns on flat surfaces, pool dynamics",
        "size_gb": "~200",
        "hf_path": "polymathic-ai/shear_flow",
    },
    {
        "name": "turbulence_gravity_cooling",
        "type": "fluid",
        "description": (
            "3D compressible turbulence with gravity and radiative cooling. "
            "Complex multi-scale fluid dynamics."
        ),
        "relevance": "High-velocity blood spatter aerodynamics, mist patterns",
        "size_gb": "~5100",
        "hf_path": "polymathic-ai/turbulence_gravity_cooling",
    },
    {
        "name": "active_matter",
        "type": "biological",
        "description": (
            "Active matter simulations of self-propelled particles. "
            "Models collective motion in biological systems."
        ),
        "relevance": "Biological fluid behaviour, blood cell dynamics",
        "size_gb": "~7",
        "hf_path": "polymathic-ai/active_matter",
    },
]


class TheWellLoader:
    """Load and query datasets from PolymathicAI/the_well.

    Focus datasets for forensics:
    - Acoustic scattering (sound propagation through complex media)
    - Navier-Stokes / shear flow (fluid dynamics reference for BPA)

    Usage::

        loader = TheWellLoader()
        if loader.available:
            ds = loader.load_dataset("acoustic_scattering_inclusions", split="train", max_samples=10)
    """

    HF_BASE_PATH = "hf://datasets/polymathic-ai/"

    def __init__(self) -> None:
        self._available = False
        self._WellDataset = None
        try:
            from the_well.data import WellDataset  # type: ignore[import-untyped]
            self._WellDataset = WellDataset
            self._available = True
            logger.info("the_well package available for physics dataset loading")
        except ImportError:
            logger.info(
                "the_well not installed (pip install the-well); "
                "physics sim will use built-in analytic models only"
            )

    @property
    def available(self) -> bool:
        """True if the the_well package is importable."""
        return self._available

    def list_relevant_datasets(self) -> list[dict[str, str]]:
        """Return the catalogue of forensics-relevant datasets.

        This is a static list — it does not require the_well to be installed.
        """
        return [
            {**ds, "installed": self._available}
            for ds in _FORENSIC_DATASETS
        ]

    def load_dataset(
        self,
        dataset_name: str,
        split: str = "train",
        max_samples: int | None = None,
    ) -> Any:
        """Load a WellDataset by name.

        Parameters
        ----------
        dataset_name : str
            One of the names from ``list_relevant_datasets()``.
        split : str
            "train", "valid", or "test".
        max_samples : int, optional
            If set, wrap in a Subset to limit memory / download.

        Returns
        -------
        WellDataset (a PyTorch map-style Dataset) or None if unavailable.
        """
        if not self._available or self._WellDataset is None:
            logger.warning("the_well not installed — cannot load dataset")
            return None

        try:
            ds = self._WellDataset(
                well_base_path=self.HF_BASE_PATH,
                well_dataset_name=dataset_name,
                well_split_name=split,
            )
            logger.info(
                "Loaded the_well dataset '{}' split='{}' ({} samples)",
                dataset_name, split, len(ds),
            )

            if max_samples is not None and max_samples < len(ds):
                from torch.utils.data import Subset  # type: ignore[import-untyped]
                ds = Subset(ds, list(range(max_samples)))
                logger.info("Subset to {} samples", max_samples)

            return ds

        except Exception as exc:
            logger.error(
                "Failed to load the_well dataset '{}': {}",
                dataset_name, exc,
            )
            return None

    def load_fluid_reference(
        self,
        scenario: str = "shear_flow",
        split: str = "train",
        max_samples: int = 10,
    ) -> Optional[dict[str, Any]]:
        """Load fluid dynamics reference data for BPA calibration.

        Returns a dict with metadata and sample tensors, or None.
        """
        ds = self.load_dataset(scenario, split=split, max_samples=max_samples)
        if ds is None:
            return None

        try:
            # Extract a few samples for inspection
            import torch  # type: ignore[import-untyped]
            samples = []
            for i in range(min(3, len(ds))):
                sample = ds[i]
                info: dict[str, Any] = {"index": i}
                if isinstance(sample, dict):
                    for k, v in sample.items():
                        if isinstance(v, torch.Tensor):
                            info[k] = {
                                "shape": list(v.shape),
                                "dtype": str(v.dtype),
                                "min": float(v.min()),
                                "max": float(v.max()),
                            }
                        else:
                            info[k] = str(v)
                else:
                    info["type"] = str(type(sample))
                samples.append(info)

            return {
                "dataset_name": scenario,
                "split": split,
                "total_samples": len(ds),
                "sample_previews": samples,
            }

        except Exception as exc:
            logger.error("Error inspecting fluid reference data: {}", exc)
            return None

    def load_acoustic_reference(
        self,
        scenario: str = "acoustic_scattering_inclusions",
        split: str = "train",
        max_samples: int = 10,
    ) -> Optional[dict[str, Any]]:
        """Load acoustic scattering reference data.

        Same structure as ``load_fluid_reference`` but defaults to
        the acoustic dataset.
        """
        return self.load_fluid_reference(
            scenario=scenario, split=split, max_samples=max_samples,
        )

    def get_dataset_info(self, dataset_name: str) -> Optional[dict[str, str]]:
        """Return metadata for a single dataset by name, or None."""
        for ds in _FORENSIC_DATASETS:
            if ds["name"] == dataset_name:
                return {**ds, "installed": self._available}
        return None
