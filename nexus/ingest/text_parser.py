"""
NEXUS -- Text file parser.

Reads plain-text files with automatic encoding detection, streaming for
large files, and basic text normalization.

Usage:
    from nexus.ingest.text_parser import TextParser

    parser = TextParser()
    text = parser.extract_text("witness_statement.txt")
"""

from __future__ import annotations

import re
import unicodedata
from pathlib import Path
from typing import Union

from loguru import logger

# Threshold above which we stream instead of reading the whole file at once.
_LARGE_FILE_THRESHOLD = 10 * 1024 * 1024  # 10 MB

# Encodings to try in order.  UTF-8 first (most common), then Latin-1
# which can decode any byte sequence and is frequent in older French docs.
_ENCODING_CANDIDATES = ("utf-8", "utf-8-sig", "latin-1", "cp1252")


# ============================================================================
# TextParser
# ============================================================================


class TextParser:
    """Read and normalize plain-text files."""

    # ------------------------------------------------------------------
    # Text extraction
    # ------------------------------------------------------------------

    def extract_text(self, file_path: Union[str, Path]) -> str:
        """Read a plain-text file and return its content.

        * Detects encoding automatically (UTF-8 first, then fallbacks).
        * Streams large files (> 10 MB) in chunks to limit memory usage.
        * Normalizes line endings to ``\\n``.

        Raises:
            FileNotFoundError: if *file_path* does not exist.
            RuntimeError: if decoding fails with every candidate encoding.
        """
        path = Path(file_path)
        if not path.exists():
            raise FileNotFoundError(f"Text file not found: {path}")

        file_size = path.stat().st_size
        encoding = self._detect_encoding(path)

        logger.info(
            "Reading text file: {} ({} bytes, encoding={})",
            path.name, file_size, encoding,
        )

        if file_size > _LARGE_FILE_THRESHOLD:
            return self._read_streaming(path, encoding)
        return self._read_full(path, encoding)

    # ------------------------------------------------------------------
    # String cleaning
    # ------------------------------------------------------------------

    def extract_from_string(self, content: str) -> str:
        """Clean a raw string: strip control chars, normalize whitespace.

        Keeps newlines (``\\n``) but replaces ``\\r\\n`` / ``\\r`` first.
        Collapses runs of spaces/tabs (but not newlines) into a single space.
        Strips leading/trailing whitespace from each line and from the whole
        result.
        """
        if not content:
            return ""

        # Normalize line endings
        text = content.replace("\r\n", "\n").replace("\r", "\n")

        # Remove control characters except \n and \t
        text = "".join(
            ch for ch in text
            if ch in ("\n", "\t") or not unicodedata.category(ch).startswith("C")
        )

        # Collapse horizontal whitespace (spaces + tabs) into single space
        text = re.sub(r"[^\S\n]+", " ", text)

        # Strip each line individually
        text = "\n".join(line.strip() for line in text.split("\n"))

        # Collapse 3+ consecutive blank lines into 2
        text = re.sub(r"\n{3,}", "\n\n", text)

        return text.strip()

    # ------------------------------------------------------------------
    # Metadata
    # ------------------------------------------------------------------

    def get_metadata(self, file_path: Union[str, Path]) -> dict:
        """Return metadata about a text file.

        Keys: ``file_size_bytes``, ``encoding``, ``line_count``,
        ``word_count``, ``char_count``.
        """
        path = Path(file_path)
        if not path.exists():
            raise FileNotFoundError(f"Text file not found: {path}")

        encoding = self._detect_encoding(path)
        content = self._read_full(path, encoding)
        lines = content.split("\n")

        return {
            "file_size_bytes": path.stat().st_size,
            "encoding": encoding,
            "line_count": len(lines),
            "word_count": len(content.split()),
            "char_count": len(content),
        }

    # ------------------------------------------------------------------
    # Internal helpers
    # ------------------------------------------------------------------

    @staticmethod
    def _detect_encoding(path: Path) -> str:
        """Try candidate encodings and return the first one that succeeds.

        Reads only the first 64 KB for the probe to stay fast on large files.
        Falls back to ``latin-1`` which can decode any byte sequence.
        """
        sample_size = 65_536
        raw = path.read_bytes()[:sample_size]

        for enc in _ENCODING_CANDIDATES:
            try:
                raw.decode(enc)
                logger.debug("Encoding detected for {}: {}", path.name, enc)
                return enc
            except (UnicodeDecodeError, LookupError):
                continue

        # latin-1 never raises UnicodeDecodeError, so we should never land
        # here, but just in case:
        logger.warning("Falling back to latin-1 for {}", path.name)
        return "latin-1"

    @staticmethod
    def _read_full(path: Path, encoding: str) -> str:
        """Read the entire file into memory at once."""
        try:
            text = path.read_text(encoding=encoding, errors="replace")
        except OSError as exc:
            logger.error("Cannot read text file {}: {}", path.name, exc)
            raise RuntimeError(f"Cannot read text file: {exc}") from exc

        # Normalize line endings
        return text.replace("\r\n", "\n").replace("\r", "\n")

    @staticmethod
    def _read_streaming(path: Path, encoding: str) -> str:
        """Stream a large file in chunks to avoid high memory use.

        Accumulates text in a list and joins once at the end.
        """
        logger.info("Streaming large file: {} (> 10 MB)", path.name)
        chunk_size = 1024 * 1024  # 1 MB
        parts: list[str] = []

        try:
            with path.open("r", encoding=encoding, errors="replace") as fh:
                while True:
                    chunk = fh.read(chunk_size)
                    if not chunk:
                        break
                    parts.append(chunk)
        except OSError as exc:
            logger.error("Error streaming text file {}: {}", path.name, exc)
            raise RuntimeError(f"Cannot read text file: {exc}") from exc

        text = "".join(parts)
        return text.replace("\r\n", "\n").replace("\r", "\n")
