"""Tests for text and PDF ingestion parsers.

TextParser tests use temp files. PDF tests only test the utility
functions (compute_file_hash, detect_mime_type) since PyMuPDF
needs real PDFs.
"""

import hashlib
import os
import tempfile
from pathlib import Path

import pytest

from nexus.ingest.text_parser import TextParser
from nexus.ingest.pdf_parser import compute_file_hash, detect_mime_type


# =====================================================================
# TextParser — extract_text
# =====================================================================


class TestTextParserExtractText:

    def test_utf8_file(self):
        parser = TextParser()
        with tempfile.NamedTemporaryFile(
            mode="w", suffix=".txt", encoding="utf-8", delete=False
        ) as f:
            f.write("Hello, world!\nLine 2.")
            f.flush()
            path = f.name
        try:
            text = parser.extract_text(path)
            assert "Hello, world!" in text
            assert "Line 2." in text
        finally:
            os.unlink(path)

    def test_latin1_file(self):
        parser = TextParser()
        with tempfile.NamedTemporaryFile(
            mode="wb", suffix=".txt", delete=False
        ) as f:
            # Write bytes that are valid latin-1 but not utf-8
            f.write("Cafe avec creme brulee\n".encode("latin-1"))
            path = f.name
        try:
            text = parser.extract_text(path)
            assert "Cafe" in text
        finally:
            os.unlink(path)

    def test_file_not_found(self):
        parser = TextParser()
        with pytest.raises(FileNotFoundError):
            parser.extract_text("/nonexistent/file.txt")

    def test_normalizes_line_endings(self):
        parser = TextParser()
        with tempfile.NamedTemporaryFile(
            mode="wb", suffix=".txt", delete=False
        ) as f:
            f.write(b"Line 1\r\nLine 2\rLine 3\n")
            path = f.name
        try:
            text = parser.extract_text(path)
            assert "\r" not in text
            assert "Line 1\nLine 2\nLine 3" in text
        finally:
            os.unlink(path)


# =====================================================================
# TextParser — extract_from_string
# =====================================================================


class TestTextParserExtractFromString:

    def test_cleans_control_chars(self):
        parser = TextParser()
        text = "Hello\x00World\x01Test"
        result = parser.extract_from_string(text)
        assert "\x00" not in result
        assert "\x01" not in result

    def test_normalizes_whitespace(self):
        parser = TextParser()
        text = "Hello   \t  World"
        result = parser.extract_from_string(text)
        assert result == "Hello World"

    def test_collapses_excessive_blank_lines(self):
        parser = TextParser()
        text = "A\n\n\n\n\nB"
        result = parser.extract_from_string(text)
        assert result == "A\n\nB"

    def test_strips_lines(self):
        parser = TextParser()
        text = "  line 1  \n  line 2  "
        result = parser.extract_from_string(text)
        assert result == "line 1\nline 2"

    def test_empty_input(self):
        parser = TextParser()
        assert parser.extract_from_string("") == ""
        assert parser.extract_from_string(None) == ""

    def test_preserves_newlines(self):
        parser = TextParser()
        text = "Line 1\nLine 2"
        result = parser.extract_from_string(text)
        assert "Line 1\nLine 2" in result

    def test_crlf_to_lf(self):
        parser = TextParser()
        text = "A\r\nB\rC"
        result = parser.extract_from_string(text)
        assert "\r" not in result
        assert "A\nB\nC" == result


# =====================================================================
# TextParser — get_metadata
# =====================================================================


class TestTextParserMetadata:

    def test_metadata(self):
        parser = TextParser()
        with tempfile.NamedTemporaryFile(
            mode="w", suffix=".txt", encoding="utf-8", delete=False
        ) as f:
            f.write("Line one\nLine two\nLine three\n")
            path = f.name
        try:
            meta = parser.get_metadata(path)
            assert meta["line_count"] == 4  # trailing newline makes 4
            assert meta["word_count"] == 6
            assert meta["encoding"] == "utf-8"
            assert meta["file_size_bytes"] > 0
        finally:
            os.unlink(path)

    def test_metadata_file_not_found(self):
        parser = TextParser()
        with pytest.raises(FileNotFoundError):
            parser.get_metadata("/nonexistent.txt")


# =====================================================================
# compute_file_hash
# =====================================================================


class TestComputeFileHash:

    def test_hash_matches_manual(self):
        content = b"Hello, this is test content for hashing."
        with tempfile.NamedTemporaryFile(delete=False) as f:
            f.write(content)
            path = f.name
        try:
            result = compute_file_hash(path)
            expected = hashlib.sha256(content).hexdigest()
            assert result == expected
        finally:
            os.unlink(path)

    def test_different_content_different_hash(self):
        with tempfile.NamedTemporaryFile(delete=False, suffix="_a") as fa:
            fa.write(b"content A")
            path_a = fa.name
        with tempfile.NamedTemporaryFile(delete=False, suffix="_b") as fb:
            fb.write(b"content B")
            path_b = fb.name
        try:
            hash_a = compute_file_hash(path_a)
            hash_b = compute_file_hash(path_b)
            assert hash_a != hash_b
        finally:
            os.unlink(path_a)
            os.unlink(path_b)

    def test_hash_length(self):
        with tempfile.NamedTemporaryFile(delete=False) as f:
            f.write(b"data")
            path = f.name
        try:
            h = compute_file_hash(path)
            assert len(h) == 64  # SHA-256 hex digest length
        finally:
            os.unlink(path)


# =====================================================================
# detect_mime_type
# =====================================================================


class TestDetectMimeType:

    def test_txt_file(self):
        with tempfile.NamedTemporaryFile(
            suffix=".txt", delete=False
        ) as f:
            f.write(b"text")
            path = f.name
        try:
            mime = detect_mime_type(path)
            assert "text" in mime
        finally:
            os.unlink(path)

    def test_pdf_extension(self):
        with tempfile.NamedTemporaryFile(
            suffix=".pdf", delete=False
        ) as f:
            f.write(b"%PDF-1.4 fake")
            path = f.name
        try:
            mime = detect_mime_type(path)
            assert mime == "application/pdf"
        finally:
            os.unlink(path)

    def test_unknown_extension_with_pdf_magic(self):
        with tempfile.NamedTemporaryFile(
            suffix=".xyz", delete=False
        ) as f:
            f.write(b"%PDF-1.7 content")
            path = f.name
        try:
            mime = detect_mime_type(path)
            assert mime == "application/pdf"
        finally:
            os.unlink(path)

    def test_unknown_extension_no_magic(self):
        with tempfile.NamedTemporaryFile(
            suffix=".xyz", delete=False
        ) as f:
            f.write(b"random bytes")
            path = f.name
        try:
            mime = detect_mime_type(path)
            assert mime == "application/octet-stream"
        finally:
            os.unlink(path)
