"""Prompts used by the gov app.

Extracted from the legacy ``nexus/engine/__init__.py``
per the Sprint 4 Phase D refactor so the core engine stays
app-agnostic. Keeping every gov-specific prompt string in this
module lets Sprint 5 delete it cleanly when the frontend stops
referencing the gov export directly.

Sprint 8 Phase D — adds ``RAG_SEARCH_PROMPT`` and
``RAG_ASK_PROMPT`` templates consumed by the new RAG workers
(``gov.rag_search`` and ``gov.rag_ask``). The workers serialize
the user query / question into the templates before forwarding
the resulting prompt to Ollama via
:meth:`nexus_sdk.ComputeClient.submit_task`.
"""

POLITICAL_CONTRADICTION_PROMPT = """\
You are an expert in detecting logical contradictions in political
statements. Given a set of statements, identify any pair that
contradicts one another, cite the exact phrases, and explain the
contradiction in plain language. Return your answer as a JSON
object with keys `contradictions` (a list of {a, b, explanation}
objects) and `confidence` (a float in [0, 1]).

Statements:
{statements}
"""


RAG_SEARCH_PROMPT = """\
You are a semantic search assistant over the gov corpus
(politicians, positions, laws, press, social posts, factchecks,
transcriptions). Given a user query, return a JSON object with
a key `matches` — a list of up to ten `{source, snippet, score}`
entries ranked by relevance. Scores are floats in [0, 1]; the
highest match comes first.

Query:
{query}
"""


RAG_ASK_PROMPT = """\
You are a factual question-answering assistant grounded in the
gov corpus (politicians, positions, laws, press, social posts,
factchecks, transcriptions). Given a user question, return a
JSON object with keys `answer` (a concise natural-language reply
citing the sources inline) and `sources` (a list of the source
URLs you used). Never invent facts — if the corpus is silent on
the question, return `answer = "Inconnu"`.

Question:
{question}
"""
