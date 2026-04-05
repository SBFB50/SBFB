"""
NEXUS -- Semantic text chunker.

Splits evidence text into overlapping chunks optimized for
embedding and retrieval. Uses recursive character splitting
with semantic boundaries (paragraphs > sentences > words).
"""

from __future__ import annotations

import re
from typing import Any


class TextChunker:
    """Split text into chunks for embedding."""

    def __init__(self, chunk_size: int = 512, overlap: int = 128):
        self.chunk_size = chunk_size  # tokens approx (1 token ~ 4 chars)
        self.overlap = overlap
        self._char_size = chunk_size * 4  # rough char equivalent
        self._char_overlap = overlap * 4

    # ------------------------------------------------------------------
    # Public API
    # ------------------------------------------------------------------

    def chunk_text(self, text: str, metadata: dict | None = None) -> list[dict]:
        """Split text into overlapping chunks with metadata.

        Strategy (recursive, semantic boundaries):
        1. Split on double newlines (paragraphs)
        2. If chunk > max, split on single newlines
        3. If still > max, split on sentences (. ! ?)
        4. If still > max, split on words

        Returns [{text, chunk_index, start_char, end_char, metadata}]
        """
        if not text or not text.strip():
            return []

        base_meta = dict(metadata or {})

        # Recursively split the text into pieces that fit within _char_size
        pieces = self._recursive_split(text)

        # Merge small pieces back together with overlap
        chunks = self._merge_with_overlap(pieces)

        # Build output dicts with positional info
        result: list[dict] = []
        search_start = 0
        for idx, chunk_text in enumerate(chunks):
            # Find the position of this chunk in the original text.
            # Overlapping chunks may not be exact substrings after merging,
            # so we search forward from the last known position.
            start_char = text.find(chunk_text[:80], search_start)
            if start_char == -1:
                # Fallback: approximate position
                start_char = search_start
            end_char = start_char + len(chunk_text)
            search_start = max(search_start, start_char + 1)

            result.append({
                "text": chunk_text,
                "chunk_index": idx,
                "start_char": start_char,
                "end_char": end_char,
                "metadata": {**base_meta, "chunk_index": idx},
            })

        return result

    def chunk_evidence(self, evidence: dict) -> list[dict]:
        """Chunk a full evidence record.

        Adds evidence metadata (id, case_id, type, source, title)
        to each chunk for filtering during retrieval.
        """
        text = evidence.get("raw_text") or evidence.get("summary") or ""
        if not text.strip():
            return []

        base_meta = {
            "evidence_id": evidence.get("id", ""),
            "case_id": evidence.get("case_id", ""),
            "evidence_type": evidence.get("evidence_type", ""),
            "source": evidence.get("source", ""),
            "title": evidence.get("title", ""),
            "reliability": evidence.get("reliability", 50),
        }

        chunks = self.chunk_text(text, metadata=base_meta)
        return chunks

    # ------------------------------------------------------------------
    # Recursive splitting
    # ------------------------------------------------------------------

    def _recursive_split(self, text: str) -> list[str]:
        """Recursively split text respecting semantic boundaries.

        Hierarchy of separators tried in order:
        1. Double newlines (paragraph breaks)
        2. Single newlines
        3. Sentence endings (. ! ? followed by space or end)
        4. Word boundaries (spaces)
        5. Hard character split (last resort)
        """
        if len(text) <= self._char_size:
            return [text]

        separators = [
            r"\n\n+",              # paragraph breaks
            r"\n",                 # line breaks
            r"(?<=[.!?])\s+",     # sentence boundaries
            r"\s+",               # word boundaries
        ]

        for sep_pattern in separators:
            pieces = self._split_by_pattern(text, sep_pattern)
            if len(pieces) > 1:
                # Recursively split any pieces that are still too large
                result: list[str] = []
                for piece in pieces:
                    if len(piece) <= self._char_size:
                        result.append(piece)
                    else:
                        # Try the next separator level on this oversized piece
                        result.extend(self._recursive_split(piece))
                return result

        # Last resort: hard split at _char_size boundaries
        return self._hard_split(text)

    def _split_by_pattern(self, text: str, pattern: str) -> list[str]:
        """Split text by regex pattern, keeping non-empty segments."""
        parts = re.split(pattern, text)
        return [p for p in parts if p.strip()]

    def _hard_split(self, text: str) -> list[str]:
        """Hard character-level split when no semantic boundary works."""
        pieces: list[str] = []
        start = 0
        while start < len(text):
            end = start + self._char_size
            if end >= len(text):
                pieces.append(text[start:])
                break
            # Try to find a space near the boundary to avoid mid-word cuts
            boundary = text.rfind(" ", start + self._char_size // 2, end)
            if boundary == -1:
                boundary = end
            pieces.append(text[start:boundary])
            start = boundary
        return [p.strip() for p in pieces if p.strip()]

    # ------------------------------------------------------------------
    # Overlap merging
    # ------------------------------------------------------------------

    def _merge_with_overlap(self, pieces: list[str]) -> list[str]:
        """Merge small pieces into chunks of ~_char_size with overlap.

        Adjacent pieces are accumulated until adding the next piece would
        exceed _char_size. Then we emit the current chunk, back up by
        _char_overlap characters worth of pieces, and continue.
        """
        if not pieces:
            return []

        chunks: list[str] = []
        current_pieces: list[str] = []
        current_len = 0

        for piece in pieces:
            piece_len = len(piece)

            # If adding this piece exceeds the limit, emit current chunk
            if current_len + piece_len > self._char_size and current_pieces:
                chunks.append(self._join_pieces(current_pieces))

                # Compute overlap: keep trailing pieces up to _char_overlap chars
                overlap_pieces: list[str] = []
                overlap_len = 0
                for prev_piece in reversed(current_pieces):
                    if overlap_len + len(prev_piece) > self._char_overlap:
                        break
                    overlap_pieces.insert(0, prev_piece)
                    overlap_len += len(prev_piece)

                current_pieces = overlap_pieces
                current_len = overlap_len

            current_pieces.append(piece)
            current_len += piece_len

        # Emit final chunk
        if current_pieces:
            chunks.append(self._join_pieces(current_pieces))

        return chunks

    @staticmethod
    def _join_pieces(pieces: list[str]) -> str:
        """Join text pieces with a single space, trimming whitespace."""
        return " ".join(p.strip() for p in pieces if p.strip())
