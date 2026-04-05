"""
Visual embeddings via DINOv2 and CLIP for image similarity search.

DINOv2 produces high-quality visual embeddings for image-to-image similarity.
CLIP produces aligned text-image embeddings for text-to-image search.

Both are stored in ChromaDB for retrieval.
"""

from __future__ import annotations

from pathlib import Path

import torch
from loguru import logger
from PIL import Image


class VisualEmbedder:
    """Generate visual embeddings using DINOv2 and/or CLIP.

    Models are loaded lazily on first use to avoid startup VRAM cost.
    Only one model is loaded at a time (VRAM management).
    """

    def __init__(self) -> None:
        self._dinov2_model = None
        self._dinov2_processor = None
        self._clip_model = None
        self._clip_processor = None
        self._device = "cuda" if torch.cuda.is_available() else "cpu"

    # ------------------------------------------------------------------
    # Model lifecycle
    # ------------------------------------------------------------------

    def _load_dinov2(self) -> None:
        """Load DINOv2 ViT-B/14 (86M params, ~350MB VRAM)."""
        if self._dinov2_model is not None:
            return
        # Unload CLIP first to free VRAM
        self._unload_clip()

        from transformers import AutoImageProcessor, AutoModel

        model_name = "facebook/dinov2-base"
        self._dinov2_processor = AutoImageProcessor.from_pretrained(model_name)
        self._dinov2_model = (
            AutoModel.from_pretrained(model_name).to(self._device).eval()
        )
        logger.info("DINOv2 loaded on {}", self._device)

    def _load_clip(self) -> None:
        """Load CLIP ViT-B/32 (150M params, ~600MB VRAM)."""
        if self._clip_model is not None:
            return
        # Unload DINOv2 first to free VRAM
        self._unload_dinov2()

        from transformers import CLIPModel, CLIPProcessor

        model_name = "openai/clip-vit-base-patch32"
        self._clip_processor = CLIPProcessor.from_pretrained(model_name)
        self._clip_model = (
            CLIPModel.from_pretrained(model_name).to(self._device).eval()
        )
        logger.info("CLIP loaded on {}", self._device)

    def _unload_dinov2(self) -> None:
        if self._dinov2_model is not None:
            del self._dinov2_model, self._dinov2_processor
            self._dinov2_model = self._dinov2_processor = None
            torch.cuda.empty_cache()
            logger.debug("DINOv2 unloaded")

    def _unload_clip(self) -> None:
        if self._clip_model is not None:
            del self._clip_model, self._clip_processor
            self._clip_model = self._clip_processor = None
            torch.cuda.empty_cache()
            logger.debug("CLIP unloaded")

    def unload_all(self) -> None:
        """Free all GPU memory held by vision models."""
        self._unload_dinov2()
        self._unload_clip()

    # ------------------------------------------------------------------
    # DINOv2 embeddings (768-dim)
    # ------------------------------------------------------------------

    @torch.no_grad()
    def embed_image_dinov2(self, image_path: str | Path) -> list[float]:
        """Embed an image using DINOv2. Returns 768-dim vector."""
        self._load_dinov2()
        image = Image.open(image_path).convert("RGB")
        inputs = self._dinov2_processor(
            images=image, return_tensors="pt"
        ).to(self._device)
        outputs = self._dinov2_model(**inputs)
        # Use CLS token as the embedding
        embedding = outputs.last_hidden_state[:, 0, :].squeeze().cpu().tolist()
        return embedding

    @torch.no_grad()
    def embed_image_batch_dinov2(
        self, image_paths: list[str | Path]
    ) -> list[list[float]]:
        """Embed multiple images with DINOv2. Returns list of 768-dim vectors."""
        self._load_dinov2()
        results: list[list[float]] = []
        for path in image_paths:
            image = Image.open(path).convert("RGB")
            inputs = self._dinov2_processor(
                images=image, return_tensors="pt"
            ).to(self._device)
            outputs = self._dinov2_model(**inputs)
            emb = outputs.last_hidden_state[:, 0, :].squeeze().cpu().tolist()
            results.append(emb)
        return results

    # ------------------------------------------------------------------
    # CLIP embeddings (512-dim, shared text-image space)
    # ------------------------------------------------------------------

    @torch.no_grad()
    def embed_image_clip(self, image_path: str | Path) -> list[float]:
        """Embed an image using CLIP. Returns 512-dim vector."""
        self._load_clip()
        image = Image.open(image_path).convert("RGB")
        inputs = self._clip_processor(
            images=image, return_tensors="pt"
        ).to(self._device)
        outputs = self._clip_model.get_image_features(**inputs)
        # L2 normalize for cosine similarity
        embedding = outputs.squeeze()
        embedding = (embedding / embedding.norm()).cpu().tolist()
        return embedding

    @torch.no_grad()
    def embed_text_clip(self, text: str) -> list[float]:
        """Embed text using CLIP. Returns 512-dim vector (same space as images)."""
        self._load_clip()
        inputs = self._clip_processor(
            text=[text],
            return_tensors="pt",
            padding=True,
            truncation=True,
        ).to(self._device)
        outputs = self._clip_model.get_text_features(**inputs)
        embedding = outputs.squeeze()
        embedding = (embedding / embedding.norm()).cpu().tolist()
        return embedding

    @torch.no_grad()
    def embed_image_batch_clip(
        self, image_paths: list[str | Path]
    ) -> list[list[float]]:
        """Embed multiple images with CLIP. Returns list of 512-dim vectors."""
        self._load_clip()
        results: list[list[float]] = []
        for path in image_paths:
            image = Image.open(path).convert("RGB")
            inputs = self._clip_processor(
                images=image, return_tensors="pt"
            ).to(self._device)
            outputs = self._clip_model.get_image_features(**inputs)
            emb = outputs.squeeze()
            emb = (emb / emb.norm()).cpu().tolist()
            results.append(emb)
        return results
