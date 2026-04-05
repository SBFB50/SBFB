"""
NEXUS -- PDF Parser.

Extracts text, metadata and images from PDF files using PyMuPDF (fitz).
Designed for the evidence ingestion pipeline.

Usage:
    from nexus.ingest.pdf_parser import PDFParser, compute_file_hash, detect_mime_type

    parser = PDFParser()
    text = parser.extract_text("report.pdf")
"""

from __future__ import annotations

import hashlib
import mimetypes
from pathlib import Path
from typing import Union

import fitz  # PyMuPDF
from loguru import logger


# ============================================================================
# Utility functions
# ============================================================================


def compute_file_hash(file_path: Union[str, Path]) -> str:
    """Compute SHA-256 hash of a file.

    Reads in 64 KB chunks to handle large files without blowing memory.
    """
    path = Path(file_path)
    sha256 = hashlib.sha256()
    try:
        with path.open("rb") as fh:
            for chunk in iter(lambda: fh.read(65_536), b""):
                sha256.update(chunk)
    except OSError as exc:
        logger.error("Cannot read file for hashing: {} -- {}", path, exc)
        raise
    return sha256.hexdigest()


def detect_mime_type(file_path: Union[str, Path]) -> str:
    """Detect MIME type of a file based on its extension.

    Falls back to ``application/octet-stream`` when the type cannot be
    determined.
    """
    path = Path(file_path)
    mime, _ = mimetypes.guess_type(str(path))
    if mime is None:
        # Try reading the first bytes for magic-number detection
        try:
            header = path.read_bytes()[:8]
            if header[:5] == b"%PDF-":
                return "application/pdf"
        except OSError:
            pass
        return "application/octet-stream"
    return mime


# ============================================================================
# PDFParser
# ============================================================================


class PDFParser:
    """Extract text, metadata and images from PDF documents."""

    # ------------------------------------------------------------------
    # Text extraction
    # ------------------------------------------------------------------

    def extract_text(self, file_path: Union[str, Path]) -> str:
        """Extract all text from a PDF, concatenating every page.

        Returns an empty string (with a warning) for scanned / image-only
        PDFs that contain no extractable text.

        Raises:
            FileNotFoundError: if *file_path* does not exist.
            ValueError: if the file is encrypted and cannot be opened.
            RuntimeError: for any other PyMuPDF failure (corrupt file, etc.).
        """
        path = Path(file_path)
        if not path.exists():
            raise FileNotFoundError(f"PDF not found: {path}")

        logger.info("Extracting text from PDF: {}", path.name)

        try:
            doc = fitz.open(str(path))
        except Exception as exc:
            logger.error("Failed to open PDF {}: {}", path.name, exc)
            raise RuntimeError(f"Cannot open PDF: {exc}") from exc

        try:
            if doc.is_encrypted:
                # Attempt to open with an empty password (owner-only protection)
                if not doc.authenticate(""):
                    logger.warning("PDF is password-protected: {}", path.name)
                    raise ValueError(f"PDF is encrypted and requires a password: {path.name}")

            pages_text: list[str] = []
            for page_num in range(doc.page_count):
                page = doc[page_num]
                text = page.get_text("text")
                if text:
                    pages_text.append(text)

            full_text = "\n".join(pages_text)
        finally:
            doc.close()

        if not full_text.strip():
            logger.warning(
                "No extractable text in PDF {} ({} pages) -- likely a scanned document",
                path.name,
                doc.page_count,
            )
            return ""

        logger.debug(
            "Extracted {} chars from {} pages of {}",
            len(full_text),
            doc.page_count,
            path.name,
        )
        return full_text

    # ------------------------------------------------------------------
    # Page-by-page extraction
    # ------------------------------------------------------------------

    def extract_text_by_page(self, file_path: Union[str, Path]) -> list[dict]:
        """Extract text from each page individually.

        Returns:
            A list of dicts ``[{"page": 1, "text": "..."}, ...]`` (1-indexed).
        """
        path = Path(file_path)
        if not path.exists():
            raise FileNotFoundError(f"PDF not found: {path}")

        logger.info("Extracting text page-by-page from: {}", path.name)

        try:
            doc = fitz.open(str(path))
        except Exception as exc:
            raise RuntimeError(f"Cannot open PDF: {exc}") from exc

        try:
            if doc.is_encrypted and not doc.authenticate(""):
                raise ValueError(f"PDF is encrypted and requires a password: {path.name}")

            results: list[dict] = []
            for page_num in range(doc.page_count):
                page = doc[page_num]
                text = page.get_text("text")
                results.append({
                    "page": page_num + 1,
                    "text": text if text else "",
                })
        finally:
            doc.close()

        return results

    # ------------------------------------------------------------------
    # Metadata
    # ------------------------------------------------------------------

    def get_metadata(self, file_path: Union[str, Path]) -> dict:
        """Return PDF metadata.

        Keys: ``title``, ``author``, ``subject``, ``creator``, ``producer``,
        ``creation_date``, ``mod_date``, ``page_count``, ``file_size_bytes``,
        ``file_hash``.
        """
        path = Path(file_path)
        if not path.exists():
            raise FileNotFoundError(f"PDF not found: {path}")

        try:
            doc = fitz.open(str(path))
        except Exception as exc:
            raise RuntimeError(f"Cannot open PDF: {exc}") from exc

        try:
            if doc.is_encrypted and not doc.authenticate(""):
                raise ValueError(f"PDF is encrypted and requires a password: {path.name}")

            raw_meta = doc.metadata or {}
            meta = {
                "title": raw_meta.get("title") or None,
                "author": raw_meta.get("author") or None,
                "subject": raw_meta.get("subject") or None,
                "creator": raw_meta.get("creator") or None,
                "producer": raw_meta.get("producer") or None,
                "creation_date": raw_meta.get("creationDate") or None,
                "mod_date": raw_meta.get("modDate") or None,
                "page_count": doc.page_count,
                "file_size_bytes": path.stat().st_size,
                "file_hash": compute_file_hash(path),
            }
        finally:
            doc.close()

        return meta

    # ------------------------------------------------------------------
    # Image extraction
    # ------------------------------------------------------------------

    def extract_images(
        self,
        file_path: Union[str, Path],
        output_dir: Union[str, Path],
    ) -> list[str]:
        """Extract all embedded images from the PDF and save them to *output_dir*.

        Each image is written as ``page{N}_img{M}.{ext}`` where N is the
        1-indexed page number and M is the image index on that page.

        Returns:
            A list of absolute paths to the saved image files.
        """
        path = Path(file_path)
        out = Path(output_dir)

        if not path.exists():
            raise FileNotFoundError(f"PDF not found: {path}")

        out.mkdir(parents=True, exist_ok=True)

        logger.info("Extracting images from PDF: {}", path.name)

        try:
            doc = fitz.open(str(path))
        except Exception as exc:
            raise RuntimeError(f"Cannot open PDF: {exc}") from exc

        saved_paths: list[str] = []

        try:
            if doc.is_encrypted and not doc.authenticate(""):
                raise ValueError(f"PDF is encrypted and requires a password: {path.name}")

            for page_num in range(doc.page_count):
                page = doc[page_num]
                image_list = page.get_images(full=True)

                for img_idx, img_info in enumerate(image_list):
                    xref = img_info[0]  # xref of the image object
                    try:
                        extracted = doc.extract_image(xref)
                    except Exception as exc:
                        logger.warning(
                            "Could not extract image xref={} on page {}: {}",
                            xref, page_num + 1, exc,
                        )
                        continue

                    if not extracted or not extracted.get("image"):
                        continue

                    ext = extracted.get("ext", "png")
                    image_bytes = extracted["image"]
                    filename = f"page{page_num + 1}_img{img_idx + 1}.{ext}"
                    dest = out / filename

                    dest.write_bytes(image_bytes)
                    saved_paths.append(str(dest.resolve()))
                    logger.debug("Saved image: {}", dest)
        finally:
            doc.close()

        logger.info(
            "Extracted {} images from {} to {}",
            len(saved_paths), path.name, out,
        )
        return saved_paths
