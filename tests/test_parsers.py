"""Tests for LLM response parsers (nexus.llm.parsers).

Covers edge cases: think tags, markdown fences, truncated JSON,
missing fields, wrong types, garbage input.
"""

import pytest

from nexus.llm.parsers import (
    clean_llm_response,
    parse_json_safe,
    parse_entities,
    parse_relations,
    parse_hypothesis_score,
    parse_verification,
    _repair_json,
)


# =====================================================================
# clean_llm_response
# =====================================================================


class TestCleanLlmResponse:

    def test_removes_think_tags(self):
        raw = "<think>Let me reason about this...</think>The answer is 42."
        assert clean_llm_response(raw) == "The answer is 42."

    def test_removes_multiline_think_tags(self):
        raw = (
            "<think>\nStep 1: consider X\n"
            "Step 2: evaluate Y\n</think>\n"
            '{"result": "ok"}'
        )
        result = clean_llm_response(raw)
        assert "<think>" not in result
        assert '{"result": "ok"}' in result

    def test_extracts_content_from_json_code_fence(self):
        raw = '```json\n{"key": "value"}\n```'
        assert clean_llm_response(raw) == '{"key": "value"}'

    def test_extracts_content_from_plain_code_fence(self):
        raw = '```\n{"a": 1}\n```'
        assert clean_llm_response(raw) == '{"a": 1}'

    def test_strips_whitespace(self):
        assert clean_llm_response("  hello  ") == "hello"

    def test_handles_both_think_and_fence(self):
        raw = (
            "<think>reasoning</think>\n"
            "```json\n"
            '{"entities": []}\n'
            "```"
        )
        result = clean_llm_response(raw)
        assert result == '{"entities": []}'

    def test_empty_input(self):
        assert clean_llm_response("") == ""

    def test_no_artifacts(self):
        plain = '{"plain": "json"}'
        assert clean_llm_response(plain) == plain


# =====================================================================
# parse_json_safe
# =====================================================================


class TestParseJsonSafe:

    def test_parses_clean_json(self):
        result = parse_json_safe('{"a": 1}')
        assert result == {"a": 1}

    def test_extracts_json_from_surrounding_text(self):
        raw = 'Here is the result: {"score": 75.0} as requested.'
        result = parse_json_safe(raw)
        assert result == {"score": 75.0}

    def test_handles_markdown_code_block(self):
        raw = '```json\n{"key": "value"}\n```'
        result = parse_json_safe(raw)
        assert result == {"key": "value"}

    def test_returns_none_on_garbage(self):
        assert parse_json_safe("this is not json at all") is None

    def test_returns_none_on_empty_string(self):
        assert parse_json_safe("") is None

    def test_handles_nested_json(self):
        raw = '{"outer": {"inner": [1, 2, 3]}}'
        result = parse_json_safe(raw)
        assert result["outer"]["inner"] == [1, 2, 3]

    def test_repairs_truncated_json_missing_brace(self):
        raw = '{"name": "John", "age": 30'
        result = parse_json_safe(raw)
        assert result is not None
        assert result["name"] == "John"

    def test_repairs_truncated_json_trailing_comma(self):
        raw = '{"items": [1, 2, 3,'
        result = parse_json_safe(raw)
        assert result is not None
        assert result["items"] == [1, 2, 3]

    def test_extracts_first_json_object(self):
        raw = 'Text before {"a": 1} text after {"b": 2}'
        result = parse_json_safe(raw)
        assert result == {"a": 1}

    def test_handles_think_tags_around_json(self):
        raw = '<think>thinking...</think>{"result": true}'
        result = parse_json_safe(raw)
        assert result == {"result": True}

    def test_json_with_unicode(self):
        raw = '{"nom": "Francois", "lieu": "Geneve"}'
        result = parse_json_safe(raw)
        assert result["nom"] == "Francois"


# =====================================================================
# _repair_json
# =====================================================================


class TestRepairJson:

    def test_closes_missing_braces(self):
        text = '{"a": {"b": 1}'
        repaired = _repair_json(text)
        assert repaired.endswith("}")

    def test_closes_missing_brackets(self):
        text = '{"items": [1, 2, 3'
        repaired = _repair_json(text)
        assert "]" in repaired and "}" in repaired

    def test_removes_trailing_comma(self):
        text = '{"a": 1, "b": 2,'
        repaired = _repair_json(text)
        assert not repaired.rstrip("}").rstrip().endswith(",")

    def test_already_valid(self):
        text = '{"a": 1}'
        assert _repair_json(text) == text


# =====================================================================
# parse_entities
# =====================================================================


class TestParseEntities:

    def test_valid_entities(self):
        raw = '{"entities": [{"name": "John", "type": "person", "confidence": 0.9, "context": "seen at bar"}]}'
        result = parse_entities(raw)
        assert len(result) == 1
        assert result[0]["name"] == "John"
        assert result[0]["type"] == "person"
        assert result[0]["confidence"] == 0.9

    def test_empty_entities_list(self):
        raw = '{"entities": []}'
        assert parse_entities(raw) == []

    def test_missing_entities_key(self):
        raw = '{"data": [{"name": "X"}]}'
        assert parse_entities(raw) == []

    def test_entities_not_a_list(self):
        raw = '{"entities": "not a list"}'
        assert parse_entities(raw) == []

    def test_filters_entries_without_name(self):
        raw = '{"entities": [{"type": "person"}, {"name": "Jane", "type": "person"}]}'
        result = parse_entities(raw)
        assert len(result) == 1
        assert result[0]["name"] == "Jane"

    def test_filters_entries_without_type(self):
        raw = '{"entities": [{"name": "Bob"}]}'
        assert parse_entities(raw) == []

    def test_normalizes_confidence(self):
        raw = '{"entities": [{"name": "A", "type": "person", "confidence": "high"}]}'
        result = parse_entities(raw)
        assert len(result) == 1
        assert result[0]["confidence"] == 0.5  # fallback

    def test_defaults_confidence_when_missing(self):
        raw = '{"entities": [{"name": "A", "type": "person"}]}'
        result = parse_entities(raw)
        assert result[0]["confidence"] == 0.5

    def test_defaults_context_when_missing(self):
        raw = '{"entities": [{"name": "A", "type": "person"}]}'
        result = parse_entities(raw)
        assert result[0]["context"] == ""

    def test_garbage_input(self):
        assert parse_entities("not json") == []

    def test_malformed_non_dict_entries(self):
        raw = '{"entities": ["string_entry", 42, {"name": "OK", "type": "person"}]}'
        result = parse_entities(raw)
        assert len(result) == 1
        assert result[0]["name"] == "OK"

    def test_entities_in_code_fence(self):
        raw = '```json\n{"entities": [{"name": "X", "type": "location"}]}\n```'
        result = parse_entities(raw)
        assert len(result) == 1


# =====================================================================
# parse_relations
# =====================================================================


class TestParseRelations:

    def test_valid_relations(self):
        raw = '{"relations": [{"source": "A", "target": "B", "type": "knows", "confidence": 0.8}]}'
        result = parse_relations(raw)
        assert len(result) == 1
        assert result[0]["source"] == "A"
        assert result[0]["target"] == "B"

    def test_empty(self):
        assert parse_relations('{"relations": []}') == []

    def test_missing_required_fields(self):
        raw = '{"relations": [{"source": "A", "type": "knows"}]}'
        result = parse_relations(raw)
        assert len(result) == 0  # missing "target"

    def test_defaults(self):
        raw = '{"relations": [{"source": "A", "target": "B", "type": "knows"}]}'
        result = parse_relations(raw)
        assert result[0]["confidence"] == 0.5
        assert result[0]["context"] == ""
        assert result[0]["temporal"] is None

    def test_garbage(self):
        assert parse_relations("nope") == []

    def test_relations_not_a_list(self):
        assert parse_relations('{"relations": "string"}') == []


# =====================================================================
# parse_hypothesis_score
# =====================================================================


class TestParseHypothesisScore:

    def test_valid(self):
        raw = """{
            "hypothesis_id": "h-1",
            "previous_score": 50.0,
            "new_score": 72.5,
            "delta": 22.5,
            "supporting": ["evidence A"],
            "contradicting": [],
            "reasoning": "New evidence corroborates",
            "status": "active"
        }"""
        result = parse_hypothesis_score(raw)
        assert result["hypothesis_id"] == "h-1"
        assert result["new_score"] == 72.5
        assert result["delta"] == 22.5

    def test_missing_fields_get_defaults(self):
        raw = '{"new_score": 60}'
        result = parse_hypothesis_score(raw)
        assert result["hypothesis_id"] == ""
        assert result["previous_score"] == 0.0
        assert result["supporting"] == []
        assert result["contradicting"] == []
        assert result["reasoning"] == ""
        assert result["status"] == "active"

    def test_non_numeric_scores_default_to_zero(self):
        raw = '{"new_score": "high", "previous_score": null}'
        result = parse_hypothesis_score(raw)
        assert result["new_score"] == 0.0
        assert result["previous_score"] == 0.0

    def test_garbage_returns_empty_dict(self):
        assert parse_hypothesis_score("not json") == {}

    def test_empty_string_returns_empty_dict(self):
        assert parse_hypothesis_score("") == {}


# =====================================================================
# parse_verification
# =====================================================================


class TestParseVerification:

    def test_valid(self):
        raw = """{
            "premises": [{"text": "A implies B", "explicit": true, "valid": true}],
            "conclusion": "Therefore B",
            "fallacies": [],
            "logical_validity": true,
            "soundness_score": 0.85,
            "critique": "Reasoning is solid."
        }"""
        result = parse_verification(raw)
        assert result["logical_validity"] is True
        assert result["soundness_score"] == 0.85

    def test_defaults(self):
        raw = "{}"
        result = parse_verification(raw)
        assert result["premises"] == []
        assert result["conclusion"] == ""
        assert result["fallacies"] == []
        assert result["logical_validity"] is False
        assert result["soundness_score"] == 0.0
        assert result["critique"] == ""

    def test_garbage(self):
        assert parse_verification("garbage") == {}

    def test_non_numeric_soundness(self):
        raw = '{"soundness_score": "high"}'
        result = parse_verification(raw)
        assert result["soundness_score"] == 0.0
