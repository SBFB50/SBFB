# Research: mouna23/OSINT-with-LLM

Repository: https://github.com/mouna23/OSINT-with-LLM
Analyzed on: 2026-04-05
Purpose: Integration documentation for the cold-case-analyst / NEXUS project

---

## 1. Full Project Architecture

```
OSINT-with-LLM/
|
|-- main.py                    # CLI entry point -- accepts email, domain, or IP as argument
|
|-- requirements.txt           # Python dependencies (pinned versions)
|
|-- README.md                  # Project documentation
|
|-- images/                    # Demo screenshots
|   |-- ll_email_1.png
|   |-- llm_domain_1.png
|   |-- llm_domain_2.png
|   |-- llm_email_2.png
|   |-- llm_ip_1.png
|   |-- llm_ip_2.png
|
|-- llm/                       # LLM integration layer (NO __init__.py)
|   |-- llm_analysis.py        # Ollama API calls + prompt templates for analysis
|
|-- recon/                     # OSINT reconnaissance modules (NO __init__.py)
|   |-- domain_recon.py        # WHOIS, Shodan, VirusTotal, SSL checks
|   |-- email_recon.py         # Breach/exposure lookup via LeakCheck
|   |-- ip_recon.py            # AbuseIPDB reputation check
|
|-- utils/                     # Utilities (NO __init__.py)
|   |-- format.py              # Input validation (email/IP/domain regex) + report saving
|
|-- reports/                   # Output directory (created at runtime)
    |-- report.md              # Generated report (overwritten each run)
```

**Notable structural issues:**
- No `__init__.py` files exist in any package directory. This works only because
  `main.py` imports directly (e.g., `from recon.domain_recon import ...`) and Python 3
  supports implicit namespace packages -- but it will cause problems with some tooling
  and test frameworks.
- Only 3 commits total in the repository. This is a proof-of-concept, not production code.
- Single contributor (mouna23), 81 stars, 11 forks as of research date.

---

## 2. Dependencies and Requirements

### requirements.txt (exact pinned versions)

```
certifi==2025.11.12
charset-normalizer==3.4.4
click==8.3.1
click-plugins==1.1.1.2
colorama==0.4.6
filelock==3.20.0
idna==3.11
requests==2.32.5
requests-file==3.0.1
shodan==1.31.0
tldextract==5.3.0
urllib3==2.5.0
whois==1.20240129.2
xlsxwriter==3.2.9
```

### Dependency Analysis

| Package          | Purpose                                          | Used In               |
|------------------|--------------------------------------------------|-----------------------|
| requests         | HTTP client for all API calls (Ollama, VT, etc.) | Everywhere            |
| shodan           | Shodan API wrapper                               | recon/domain_recon.py |
| whois            | WHOIS domain lookup                              | recon/domain_recon.py |
| tldextract       | Domain parsing (listed but not imported in code)  | Unused                |
| xlsxwriter       | Excel report generation (listed but not used)     | Unused                |
| colorama         | Terminal color output (transitive dep)            | Not directly used     |
| click            | CLI framework (transitive dep of shodan)          | Not directly used     |
| certifi, charset-normalizer, idna, urllib3 | Transitive deps of requests    | N/A                   |
| requests-file    | File:// URI support for requests                  | Unclear               |
| filelock         | File locking (transitive dep)                     | Not directly used     |

### Unlisted Dependencies (required but NOT in requirements.txt)

The code uses these Python standard library modules that need no installation:
- `ssl` (SSL certificate checking)
- `socket` (TCP connections for SSL check)
- `datetime` (Certificate expiry calculation)
- `os` (Directory creation for reports)
- `re` (Input validation regex)
- `sys` (CLI argument parsing)

### External System Dependencies (not in requirements.txt)

1. **Ollama** -- must be installed and running locally on port 11434
2. **Mistral model** -- must be pulled into Ollama (`ollama pull mistral`)
3. **Python 3.13+** -- per the README, though the code itself has no 3.13-specific features

### API Keys Required

These are hardcoded as placeholder strings in the source code (not via environment variables, despite what the README implies):

| API Key         | Where Used              | Hardcoded Location                  |
|-----------------|-------------------------|-------------------------------------|
| Shodan          | domain_recon.py line 15 | `shodan.Shodan("your-api-key")`     |
| VirusTotal      | domain_recon.py line 27 | `headers = {"x-apikey": "your-api-key"}` |
| AbuseIPDB       | ip_recon.py line 5      | `headers = {"Key": "your-api-key"}` |

**Critical issue:** The README says to set `VT_API_KEY`, `ABUSEIPDB_KEY`, and `SHODAN_KEY`
as environment variables, but the actual code does NOT read from environment variables. The
keys are hardcoded as literal `"your-api-key"` strings. Users must edit the source files
directly or refactor to use `os.environ.get()`.

---

## 3. How It Connects to Ollama

### Connection Mechanism

The entire Ollama integration lives in a single file: `llm/llm_analysis.py`.

The function `ask_llm()` makes a direct HTTP POST to the Ollama REST API:

```python
def ask_llm(prompt, model="mistral"):
    res = requests.post("http://localhost:11434/api/generate", json={
        "model": model,
        "prompt": prompt,
        "stream": False
    })
    output = res.json().get("response", "[No response]")
    return output.strip()
```

### Key Details

- **Endpoint:** `http://localhost:11434/api/generate`
- **HTTP Method:** POST
- **Payload format:** `{"model": "<name>", "prompt": "<text>", "stream": false}`
- **Default model:** `mistral` (hardcoded as function parameter default)
- **Streaming:** Disabled (`"stream": False`) -- waits for complete response
- **Response parsing:** Extracts `response` field from JSON reply
- **Error handling:** Basic try/except, returns `"[LLM Error]"` or `"[LLM Exception]"`
- **No timeout:** The `requests.post()` call has no timeout parameter, so it will hang
  indefinitely if Ollama is slow or unresponsive
- **No authentication:** Direct localhost connection, no API key needed for Ollama
- **No conversation context:** Each call is stateless -- no system prompt via the API,
  no message history, no chat endpoint. Uses the older `/api/generate` endpoint, not
  `/api/chat`

### What Is NOT Configurable

- The Ollama host/port is hardcoded (`localhost:11434`)
- The model name is hardcoded as `"mistral"` default
- No system prompt is sent via the API (role instructions are embedded in the user prompt)
- No temperature, top_p, num_ctx, or other generation parameters are set
- No support for Ollama's `/api/chat` endpoint (which supports message roles)

---

## 4. OSINT Reconnaissance Modules

### 4.1 Domain Recon (recon/domain_recon.py)

Four functions, each targeting a different data source:

#### a) WHOIS Lookup -- `get_whois(domain)`
- Uses the `whois` Python library
- Returns raw WHOIS text (registrant, registrar, dates, nameservers)
- Basic error handling: returns error string on failure

#### b) Shodan Search -- `search_shodan(domain)`
- Uses the `shodan` Python library (Shodan API wrapper)
- Calls `api.search(domain)` which searches Shodan for the domain
- Returns top 5 matches with: IP address, port, organization
- API key is hardcoded as `"your-api-key"`
- Bare `except:` clause (catches and silences all errors)

#### c) VirusTotal Domain Check -- `check_domain_virustotal(domain)`
- Direct HTTP GET to `https://www.virustotal.com/api/v3/domains/{domain}`
- Parses `last_analysis_stats` from response
- Returns simple string: `"domain is malicious"` or `"domain is clean"`
- Checks both `malicious` and `suspicious` counters
- API key is hardcoded

#### d) SSL Certificate Check -- `check_ssl(domain)`
- Uses Python standard library (`ssl`, `socket`)
- Connects to port 443, retrieves peer certificate
- Calculates days until expiration
- Returns dict with: status, issuer, subject, expire_date, days_left
- Returns `None` (implicitly) on any error (bare `except:`)

#### Domain Data Aggregation (in main.py)

`get_domain_data(domain)` calls all four functions and concatenates results into a
single raw text block:

```
WHOIS:
[whois data]

Shodan:
[shodan data]

SSL check:
[ssl data]

Virustotal status:
[vt status]
```

This raw text is then passed to the LLM for analysis.

### 4.2 IP Recon (recon/ip_recon.py)

Single function:

#### AbuseIPDB Check -- `check_ip(ip)`
- HTTP GET to `https://api.abuseipdb.com/api/v2/check`
- Parameters: IP address, 90-day lookback window
- Returns raw JSON response (abuse confidence score, report count, categories)
- API key is hardcoded

### 4.3 Email Recon (recon/email_recon.py)

Single function:

#### LeakCheck Breach Lookup -- `search_breaches(email)`
- HTTP GET to `https://leakcheck.io/api/public?check={email}`
- Returns raw JSON response (breach sources, exposed data types)
- No API key required (uses public endpoint)
- No error handling whatsoever

### Summary of OSINT Capabilities

| Target Type | Data Source   | Information Gathered                            |
|-------------|---------------|-------------------------------------------------|
| Domain      | WHOIS         | Registrant, registrar, creation/expiry dates    |
| Domain      | Shodan        | Open ports, IPs, organizations                  |
| Domain      | VirusTotal    | Malicious/clean classification                  |
| Domain      | SSL/TLS       | Certificate issuer, subject, expiry             |
| IP          | AbuseIPDB     | Abuse score, report count, malicious categories |
| Email       | LeakCheck     | Breach history, exposed data sources            |

---

## 5. How Reports Are Generated

### Flow

```
User Input (CLI argument)
    |
    v
Input Validation (utils/format.py -- regex matching)
    |
    v
[is_email?] --> get_email_data() --> search_breaches()     --> summarize_email()
[is_ip?]    --> get_ip_data()    --> check_ip()            --> summarize_ip()
[is_domain?]--> get_domain_data()--> whois+shodan+vt+ssl   --> summarize_domain()
    |
    v
Raw OSINT data concatenated as plain text
    |
    v
LLM prompt constructed (role + instructions + raw data)
    |
    v
ask_llm() sends prompt to Ollama (Mistral model)
    |
    v
LLM returns human-readable analysis text
    |
    v
save_report() writes to reports/report.md (UTF-8)
```

### LLM Prompt Templates

Each target type has a specialized system-role prompt embedded in the user message:

**Domain analysis prompt:**
```
You are an OSINT analyst.
Analyze this domain data and summarize key security findings:
- WHOIS or registrant issues
- Subdomain risks
- Is it malicious or not based on virustotal result
- Action recommendations

DATA:
{raw_data}
```

**Email analysis prompt:**
```
You are a breach analyst.
Summarize this email breach data:
- Sources of exposure
- Likely leaked data types
- Risk level
- Remediation advice

DATA:
{raw_data}
```

**IP analysis prompt:**
```
You are a SOC (Security Operations Center) analyst.
Summarize this IP intelligence report:
- Whether the IP is malicious
- Number of abuse reports
- Type of malicious activity
- Action recommendations

DATA:
{raw_data}
```

### Report Output

- Single file: `reports/report.md`
- Overwritten on every run (no history, no timestamping, no unique filenames)
- Content is the raw LLM text output -- no post-processing, no structured format
- Directory `reports/` is created automatically via `os.makedirs(..., exist_ok=True)`
- Encoding: UTF-8

---

## 6. API Endpoints

### This project has NO web API

OSINT-with-LLM is a **CLI-only tool**. There is no Flask, FastAPI, Django, or any other
web framework. There are no HTTP endpoints exposed by the project itself.

The project **consumes** the following external APIs:

| API                | Endpoint                                              | Method | Auth          |
|--------------------|-------------------------------------------------------|--------|---------------|
| Ollama (local)     | `http://localhost:11434/api/generate`                 | POST   | None          |
| VirusTotal         | `https://www.virustotal.com/api/v3/domains/{domain}`  | GET    | x-apikey header|
| AbuseIPDB          | `https://api.abuseipdb.com/api/v2/check`              | GET    | Key header    |
| Shodan             | Via `shodan` Python library (wraps REST API)          | --     | API key       |
| LeakCheck          | `https://leakcheck.io/api/public?check={email}`      | GET    | None (public) |

---

## 7. Limitations and Issues

### Critical Issues

1. **Hardcoded API keys as placeholders** -- The README says to set environment variables
   (`VT_API_KEY`, `ABUSEIPDB_KEY`, `SHODAN_KEY`) but the code uses literal
   `"your-api-key"` strings. The code will fail at runtime unless the user edits the
   source files directly.

2. **No `__init__.py` files** -- The `llm/`, `recon/`, and `utils/` directories lack
   `__init__.py` files. This relies on Python 3's implicit namespace packages, which
   can cause import failures in certain contexts (testing, packaging, some IDEs).

3. **Report overwrite** -- Every run overwrites `reports/report.md` with no history.
   Previous analysis is permanently lost.

4. **No request timeouts** -- The Ollama API call (`requests.post`) and all external
   API calls have no timeout set. A non-responsive service will hang the tool
   indefinitely.

5. **Silent error swallowing** -- Multiple bare `except:` clauses (Shodan, SSL check)
   catch all exceptions and return generic strings or `None`, making debugging impossible.

### Functional Limitations

6. **Single-target only** -- Can analyze one target per run. No batch processing, no
   pipeline mode.

7. **No data persistence** -- Raw OSINT data is not saved, only the LLM summary.
   The original API responses are lost after each run.

8. **Stateless LLM calls** -- Uses `/api/generate` (not `/api/chat`), sends no system
   message, maintains no conversation history. Each analysis is completely isolated.

9. **No streaming** -- `"stream": False` means the entire response must be generated
   before any output appears. For large analyses this creates long silent waits.

10. **Mistral-only** -- Model is hardcoded as default parameter. No configuration file
    or CLI flag to switch models.

11. **LeakCheck public API limitations** -- The public endpoint has severe rate limits
    and returns limited data. The paid API would be needed for real use.

12. **No input sanitization beyond regex** -- Target strings are passed directly into
    URLs and API calls. While the regex validation provides basic filtering, there is
    no explicit sanitization against injection.

### Code Quality Issues

13. **No tests** -- Zero test files in the repository.
14. **No logging** -- Uses `print()` statements, no structured logging.
15. **No configuration file** -- All settings (API keys, model name, Ollama URL, output
    path) are hardcoded in source files.
16. **No virtual environment or containerization** -- No Dockerfile, no docker-compose,
    no pyproject.toml.
17. **`xlsxwriter` is unused** -- Listed in requirements.txt but never imported.
    Possibly planned for future Excel report output.
18. **`tldextract` is unused** -- Listed in requirements.txt but never imported.

---

## 8. Integration with NEXUS (Gemma 4 26B Heretic via Open WebUI + SearXNG)

### 8.1 Current Architecture Comparison

| Aspect               | OSINT-with-LLM (upstream)           | NEXUS (cold-case-analyst)                   |
|----------------------|--------------------------------------|---------------------------------------------|
| LLM Backend          | Ollama, Mistral model                | Ollama, Gemma 4 26B Heretic (uncensored)    |
| Ollama Endpoint      | localhost:11434/api/generate         | localhost:11434/api/generate                 |
| Model Name           | `mistral`                            | `nexus`                                     |
| System Prompt        | Embedded in user prompt              | In Modelfile (SYSTEM directive)             |
| Web Search           | None                                 | SearXNG on port 8888                        |
| UI                   | CLI only                             | Open WebUI                                  |
| Context Window       | Ollama defaults (~4K for Mistral)    | 32768 tokens (explicit in Modelfile)        |
| Temperature          | Ollama defaults (~0.8)               | 0.3 (conservative, analytical)              |
| Use Case             | Generic security OSINT               | Cold case criminal investigation            |

### 8.2 Integration Strategy: Minimal Changes

The simplest integration path requires modifying only `llm/llm_analysis.py` to use the
NEXUS model instead of Mistral.

#### Change 1: Model name

```python
# BEFORE
def ask_llm(prompt, model="mistral"):

# AFTER
def ask_llm(prompt, model="nexus"):
```

This single change routes all LLM calls to your custom Gemma 4 26B Heretic model. Since
both use the same Ollama `/api/generate` endpoint on localhost:11434, no other connection
changes are needed.

#### Change 2: Add generation parameters (recommended)

The upstream code sends no generation parameters. NEXUS should enforce its own:

```python
def ask_llm(prompt, model="nexus"):
    res = requests.post("http://localhost:11434/api/generate", json={
        "model": model,
        "prompt": prompt,
        "stream": False,
        "options": {
            "temperature": 0.3,
            "top_p": 0.9,
            "num_ctx": 32768,
            "repeat_penalty": 1.1
        }
    }, timeout=300)
```

These match the parameters from `Modelfile.gemma4-heretic`. The `timeout=300` (5 minutes)
prevents indefinite hangs -- Gemma 4 26B will be slower than Mistral 7B.

### 8.3 Integration Strategy: Using /api/chat Instead of /api/generate

The upstream project uses the older `/api/generate` endpoint which does not support
message roles. Your NEXUS Modelfile already defines a detailed SYSTEM prompt. To
leverage it properly, switch to `/api/chat`:

```python
def ask_llm(prompt, model="nexus"):
    res = requests.post("http://localhost:11434/api/chat", json={
        "model": model,
        "messages": [
            {
                "role": "user",
                "content": prompt
            }
        ],
        "stream": False,
        "options": {
            "temperature": 0.3,
            "top_p": 0.9,
            "num_ctx": 32768,
            "repeat_penalty": 1.1
        }
    }, timeout=300)
    output = res.json().get("message", {}).get("content", "[No response]")
    return output.strip()
```

With `/api/chat`, the SYSTEM prompt from the Modelfile is automatically injected by
Ollama. The role-based prompts in `llm_analysis.py` ("You are an OSINT analyst", etc.)
become the user message, and NEXUS's 5-phase analytical methodology is applied
automatically via the system prompt.

**Important:** The response format differs between endpoints:
- `/api/generate` returns `{"response": "..."}`
- `/api/chat` returns `{"message": {"role": "assistant", "content": "..."}}`

### 8.4 Integration Strategy: Adding SearXNG Web Search

Your cold-case-analyst project has SearXNG running on port 8888. The upstream OSINT
project does NOT use web search at all -- it only calls specific APIs (Shodan, VT, etc.).

You can add SearXNG as an additional recon source. Create a new module:

```python
# recon/web_recon.py

import requests

SEARXNG_URL = "http://localhost:8888/search"

def search_web(query, categories="general", language="fr", max_results=10):
    """Query SearXNG for OSINT enrichment."""
    try:
        params = {
            "q": query,
            "format": "json",
            "categories": categories,
            "language": language,
        }
        r = requests.get(SEARXNG_URL, params=params, timeout=30)
        if r.status_code != 200:
            return f"SearXNG error: {r.status_code}"

        data = r.json()
        results = []
        for item in data.get("results", [])[:max_results]:
            results.append({
                "title": item.get("title", ""),
                "url": item.get("url", ""),
                "content": item.get("content", ""),
                "engine": item.get("engine", ""),
            })
        return results
    except Exception as e:
        return f"SearXNG exception: {e}"


def search_person(name, context=""):
    """Search for a person across web sources."""
    query = f"{name} {context}".strip()
    return search_web(query, categories="general")


def search_news(topic, language="fr"):
    """Search news sources for a topic."""
    return search_web(topic, categories="news", language=language)
```

Then integrate it into `main.py` by adding web search enrichment before or after the
existing OSINT modules run.

### 8.5 Integration Strategy: Open WebUI Compatibility

Open WebUI provides its own web interface and can connect to Ollama directly. There
are two integration paths:

**Path A: Use OSINT-with-LLM as a backend data collector, Open WebUI as the analysis UI**

1. Run the OSINT recon modules to collect raw data (WHOIS, Shodan, VT, AbuseIPDB, etc.)
2. Save the raw data to a file
3. Paste the raw data into Open WebUI's chat interface where NEXUS is already loaded
4. Let NEXUS analyze it with its full system prompt and SearXNG web search capability

This keeps the two systems separate and uses each for its strength.

**Path B: Build a unified pipeline**

1. Modify OSINT-with-LLM to save raw OSINT data as structured JSON
2. Add a SearXNG enrichment step
3. Feed the combined data to NEXUS via the Ollama API
4. Save the report with timestamp and target identifier

### 8.6 Recommended Rewrite of llm/llm_analysis.py for NEXUS

```python
# llm/llm_analysis.py -- NEXUS integration

import requests
import json

OLLAMA_URL = "http://localhost:11434/api/chat"
OLLAMA_MODEL = "nexus"
OLLAMA_TIMEOUT = 600  # 10 minutes -- Gemma 4 26B is slower than Mistral 7B
OLLAMA_OPTIONS = {
    "temperature": 0.3,
    "top_p": 0.9,
    "num_ctx": 32768,
    "repeat_penalty": 1.1,
}


def ask_llm(prompt, model=None):
    """Send a prompt to NEXUS via Ollama /api/chat endpoint."""
    model = model or OLLAMA_MODEL
    print(f"[NEXUS] Sending prompt to {model}...")
    try:
        res = requests.post(OLLAMA_URL, json={
            "model": model,
            "messages": [
                {"role": "user", "content": prompt}
            ],
            "stream": False,
            "options": OLLAMA_OPTIONS,
        }, timeout=OLLAMA_TIMEOUT)

        if res.status_code != 200:
            print(f"[NEXUS] Error: {res.status_code} - {res.text}")
            return "[NEXUS Error]"

        output = res.json().get("message", {}).get("content", "[No response]")
        return output.strip()

    except requests.exceptions.Timeout:
        print("[NEXUS] Request timed out")
        return "[NEXUS Timeout]"
    except Exception as e:
        print(f"[NEXUS] Exception: {e}")
        return "[NEXUS Exception]"


def summarize_domain(raw_data):
    prompt = f"""OSINT domain reconnaissance data is provided below.
Analyze using your full methodology. Identify:
- WHOIS registrant anomalies
- Infrastructure risks from Shodan results
- VirusTotal threat classification
- SSL/TLS certificate issues
- Cross-correlations between data sources
- Risk assessment with confidence score
- Prioritized investigation actions

DATA:
{raw_data}
"""
    return ask_llm(prompt)


def summarize_email(raw_data):
    prompt = f"""OSINT email breach data is provided below.
Analyze using your full methodology. Identify:
- All breach sources and their dates
- Types of data exposed (passwords, personal info, financial)
- Cross-correlation of breach sources
- Risk level assessment with confidence score
- Whether this email links to other compromised accounts
- Remediation and investigation actions

DATA:
{raw_data}
"""
    return ask_llm(prompt)


def summarize_ip(raw_data):
    prompt = f"""OSINT IP intelligence data is provided below.
Analyze using your full methodology. Identify:
- Malicious activity classification
- Abuse report volume and patterns
- Associated threat actors or campaigns
- Geolocation and ISP context
- Risk assessment with confidence score
- Recommended defensive and investigation actions

DATA:
{raw_data}
"""
    return ask_llm(prompt)
```

### 8.7 Environment Variables Refactor for API Keys

The hardcoded API keys should be extracted to environment variables. Here is the
refactored approach for `recon/domain_recon.py` and `recon/ip_recon.py`:

```python
import os

# In domain_recon.py
SHODAN_KEY = os.environ.get("SHODAN_KEY", "")
VT_API_KEY = os.environ.get("VT_API_KEY", "")

# In ip_recon.py
ABUSEIPDB_KEY = os.environ.get("ABUSEIPDB_KEY", "")
```

Or use a single `.env` file with `python-dotenv`:

```
# .env
SHODAN_KEY=your_actual_key
VT_API_KEY=your_actual_key
ABUSEIPDB_KEY=your_actual_key
OLLAMA_MODEL=nexus
OLLAMA_URL=http://localhost:11434
SEARXNG_URL=http://localhost:8888
```

### 8.8 Complete Integration Architecture

```
                     +------------------+
                     |   Open WebUI     |  (web interface, manual analysis)
                     |   + SearXNG      |  (web search for NEXUS)
                     +--------+---------+
                              |
                              v
                     +------------------+
                     |     Ollama       |  localhost:11434
                     |  model: nexus    |  Gemma 4 26B Heretic (uncensored)
                     |  (32K context)   |
                     +--------+---------+
                              ^
                              |
              +---------------+----------------+
              |                                |
    +---------+----------+        +-----------+-----------+
    | OSINT-with-LLM     |        | Manual paste into     |
    | (automated recon)   |        | Open WebUI chat       |
    |                     |        | (interactive analysis) |
    | recon/              |        +-----------------------+
    |  domain_recon.py    |
    |  ip_recon.py        |
    |  email_recon.py     |
    |  web_recon.py (new) +----> SearXNG localhost:8888
    |                     |
    | llm/                |
    |  llm_analysis.py    +----> Ollama /api/chat (nexus)
    |                     |
    | reports/            |
    |  report.md          |
    +---------+-----------+
              |
              v
    +-------------------+
    | Structured Output |  (JSON + Markdown reports)
    | for NEXUS pipeline|  (cold case data ingestion)
    +-------------------+
```

### 8.9 Key Integration Considerations

1. **Response time**: Gemma 4 26B (Q4_K_S quantized) will be significantly slower than
   Mistral 7B. Budget 2-10 minutes per OSINT analysis depending on hardware. Set
   appropriate timeouts.

2. **Context window**: The upstream project sends raw OSINT data directly in the prompt.
   With NEXUS's 32K context window, this is fine for single-target recon. But if you
   combine multiple data sources or batch targets, you could exceed the context. Monitor
   prompt size.

3. **System prompt conflict**: The upstream prompts say "You are an OSINT analyst" / "You
   are a breach analyst" / "You are a SOC analyst". Your NEXUS Modelfile defines the
   model as a cold case criminal analyst. When using `/api/chat`, the Modelfile SYSTEM
   prompt takes precedence as the base identity. The OSINT role prompts become user
   instructions within that identity. This is actually beneficial -- NEXUS will analyze
   OSINT data through its criminal investigation lens.

4. **Uncensored model advantage**: The Heretic variant of Gemma 4 will not refuse to
   analyze sensitive data (breach information, abuse reports, malicious infrastructure).
   This is important for legitimate OSINT work where standard models might refuse.

5. **SearXNG enrichment**: The upstream project's OSINT is limited to 4 API sources.
   SearXNG adds broad web search capability. Particularly valuable for:
   - Cross-referencing domain/IP with news articles about cyber incidents
   - Finding social media profiles linked to email addresses
   - Discovering public records related to WHOIS registrants
   - Identifying related criminal cases (modus operandi matching)

6. **Open WebUI as alternative interface**: Instead of CLI-only operation, analysts can
   paste OSINT results into Open WebUI for interactive follow-up questions. This
   enables the "reinvestigation continue" workflow from your prompts.

7. **Report format alignment**: The upstream saves raw LLM text to `report.md`. Your
   NEXUS system prompt defines a structured report format (with headers, scoring,
   hypothesis ranking). When NEXUS generates OSINT reports, they will automatically
   follow the cold case analysis structure, which is more thorough than the upstream's
   unstructured output.
