"""
NEXUS -- Ingestion layer.

Parsers for extracting text and metadata from evidence files.
"""

from nexus.ingest.pdf_parser import PDFParser, compute_file_hash, detect_mime_type
from nexus.ingest.text_parser import TextParser

__all__ = [
    "PDFParser",
    "TextParser",
    "compute_file_hash",
    "detect_mime_type",
]
