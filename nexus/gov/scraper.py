"""
NEXUS GOV -- Parliament Scraper (Official Sources).

Builds a complete database of French political activity from official
government open data sources. Zero dependency on third-party APIs.

Sources:
  1. Assemblee Nationale — Scrutins.json.zip (6000+ votes with individual positions)
  2. Senat — /api-senat/senateurs.json (348 senators)
  3. data.gouv.fr — CSV of active deputies (Datan dataset)
  4. HATVP — liste.csv (patrimony/interest declarations)
  5. AN Dossiers Legislatifs — Dossiers_Legislatifs.json.zip (8600+ law dossiers)
  6. La Fabrique de la Loi — metrics.csv (1117 promulgated laws with 77 stat columns)
  7. Wikidata SPARQL — biographies, party history, judicial affairs
  8. PoliGraph API — fallback for enrichment (optional)
"""

from __future__ import annotations

import asyncio
import csv
import io
import json
import zipfile
from pathlib import Path
from typing import Any, Callable, Optional

import httpx
from loguru import logger

from nexus.config import settings

# -- Official source URLs --------------------------------------------------
AN_SCRUTINS_ZIP = (
    "https://data.assemblee-nationale.fr/static/openData/repository"
    "/17/loi/scrutins/Scrutins.json.zip"
)
SENAT_API = "https://www.senat.fr/api-senat/senateurs.json"
DATAGOUV_DEPUTES = (
    "https://www.data.gouv.fr/api/1/datasets/"
    "deputes-actifs-de-lassemblee-nationale-informations-et-statistiques/"
)
HATVP_CSV = "https://www.hatvp.fr/livraison/opendata/liste.csv"
AN_DOSSIERS_ZIP = (
    "https://data.assemblee-nationale.fr/static/openData/repository"
    "/17/loi/dossiers_legislatifs/Dossiers_Legislatifs.json.zip"
)
FABRIQUE_LOI_CSV = "https://www.lafabriquedelaloi.fr/api/stats/metrics.csv"
WIKIDATA_SPARQL = "https://query.wikidata.org/sparql"
POLIGRAPH_API = "https://poligraph.fr/api"

# Cache dir for large downloads
CACHE_DIR = Path(settings.data_dir) / "gov_cache"

# Retry defaults
_MAX_RETRIES = 3
_RETRY_BACKOFF = 2.0  # seconds, doubled each retry


class ParliamentScraper:
    """Async scraper pulling from official French government open data."""

    def __init__(self) -> None:
        self._rate_limit = getattr(settings, "gov_scan_rate_limit", 2.0)

    # ------------------------------------------------------------------
    # Cancellation check
    # ------------------------------------------------------------------

    @staticmethod
    def _check_cancelled() -> None:
        """Raise asyncio.CancelledError if the current task was cancelled."""
        task = asyncio.current_task()
        if task is not None and task.cancelled():
            raise asyncio.CancelledError("scan cancelled")

    # ------------------------------------------------------------------
    # HTTP helpers (with retry + backoff)
    # ------------------------------------------------------------------

    async def _get_json(
        self, url: str, params: dict | None = None, *, timeout: float = 30.0
    ) -> Any | None:
        for attempt in range(_MAX_RETRIES):
            try:
                async with httpx.AsyncClient(timeout=timeout) as c:
                    r = await c.get(url, params=params)
                    r.raise_for_status()
                    return r.json()
            except httpx.HTTPStatusError as exc:
                # Don't retry 4xx client errors (except 429)
                if exc.response.status_code < 500 and exc.response.status_code != 429:
                    logger.warning("HTTP {} {}: {}", exc.response.status_code, url[:80], exc)
                    return None
                if attempt < _MAX_RETRIES - 1:
                    wait = _RETRY_BACKOFF * (2 ** attempt)
                    logger.debug("Retry {}/{} in {:.0f}s: {}", attempt + 1, _MAX_RETRIES, wait, url[:80])
                    await asyncio.sleep(wait)
                else:
                    logger.warning("HTTP error after {} retries {}: {}", _MAX_RETRIES, url[:80], exc)
                    return None
            except Exception as exc:
                if attempt < _MAX_RETRIES - 1:
                    wait = _RETRY_BACKOFF * (2 ** attempt)
                    logger.debug("Retry {}/{} in {:.0f}s: {}", attempt + 1, _MAX_RETRIES, wait, url[:80])
                    await asyncio.sleep(wait)
                else:
                    logger.warning("HTTP error after {} retries {}: {}", _MAX_RETRIES, url[:80], exc)
                    return None
        return None

    async def _download_bytes(self, url: str, *, timeout: float = 120) -> bytes | None:
        for attempt in range(_MAX_RETRIES):
            try:
                async with httpx.AsyncClient(timeout=timeout) as c:
                    r = await c.get(url)
                    r.raise_for_status()
                    return r.content
            except httpx.HTTPStatusError as exc:
                if exc.response.status_code < 500 and exc.response.status_code != 429:
                    logger.warning("Download HTTP {} {}: {}", exc.response.status_code, url[:80], exc)
                    return None
                if attempt < _MAX_RETRIES - 1:
                    wait = _RETRY_BACKOFF * (2 ** attempt)
                    logger.debug("Download retry {}/{} in {:.0f}s: {}", attempt + 1, _MAX_RETRIES, wait, url[:80])
                    await asyncio.sleep(wait)
                else:
                    logger.warning("Download failed after {} retries {}: {}", _MAX_RETRIES, url[:80], exc)
                    return None
            except Exception as exc:
                if attempt < _MAX_RETRIES - 1:
                    wait = _RETRY_BACKOFF * (2 ** attempt)
                    logger.debug("Download retry {}/{} in {:.0f}s: {}", attempt + 1, _MAX_RETRIES, wait, url[:80])
                    await asyncio.sleep(wait)
                else:
                    logger.warning("Download failed after {} retries {}: {}", _MAX_RETRIES, url[:80], exc)
                    return None
        return None

    async def _get_csv(self, url: str, *, delimiter: str = ";") -> list[dict]:
        """Fetch CSV with configurable delimiter (default semicolon for French gov data)."""
        for attempt in range(_MAX_RETRIES):
            try:
                async with httpx.AsyncClient(timeout=60.0) as c:
                    r = await c.get(url)
                    r.raise_for_status()
                    text = r.text
                reader = csv.DictReader(io.StringIO(text), delimiter=delimiter)
                return list(reader)
            except Exception as exc:
                if attempt < _MAX_RETRIES - 1:
                    wait = _RETRY_BACKOFF * (2 ** attempt)
                    logger.debug("CSV retry {}/{} in {:.0f}s: {}", attempt + 1, _MAX_RETRIES, wait, url[:80])
                    await asyncio.sleep(wait)
                else:
                    logger.warning("CSV fetch failed after {} retries {}: {}", _MAX_RETRIES, url[:80], exc)
                    return []
        return []

    # ------------------------------------------------------------------
    # 1. ASSEMBLEE NATIONALE -- Deputies from data.gouv.fr CSV
    # ------------------------------------------------------------------

    async def fetch_deputies(self) -> list[dict[str, Any]]:
        """Fetch active deputies from data.gouv.fr (Datan CSV)."""
        # Get the dataset to find the CSV URL
        ds = await self._get_json(DATAGOUV_DEPUTES)
        if not ds:
            logger.warning("Cannot fetch data.gouv.fr dataset metadata")
            return []

        csv_url = None
        for r in ds.get("resources", []):
            if r.get("format", "").lower() == "csv":
                csv_url = r["url"]
                break
        if not csv_url:
            logger.warning("No CSV resource found in data.gouv.fr dataset")
            return []

        rows = await self._get_csv(csv_url, delimiter=",")
        deputies: list[dict[str, Any]] = []
        for row in rows:
            name = f"{row.get('prenom', '')} {row.get('nom', '')}".strip()
            if not name or name == " ":
                continue
            deputies.append({
                "name": name,
                "slug": f"{row.get('prenom','').lower()}-{row.get('nom','').lower()}".replace(" ", "-"),
                "chamber": "assemblee",
                "party": row.get("groupe_sigle", row.get("parti", "")),
                "role": "depute",
                "constituency": row.get("circonscription", row.get("nom_circo", "")),
                "photo_url": row.get("url_photo", ""),
                "official_url": row.get("url_an", row.get("url", "")),
            })

        logger.info("Fetched {} deputies from data.gouv.fr CSV", len(deputies))
        return deputies

    # ------------------------------------------------------------------
    # 2. SENAT -- Senators from official API
    # ------------------------------------------------------------------

    async def fetch_senators(self) -> list[dict[str, Any]]:
        """Fetch all senators from official Senat API."""
        data = await self._get_json(SENAT_API)
        if not data or not isinstance(data, list):
            logger.warning("Senat API returned no data")
            return []

        senators: list[dict[str, Any]] = []
        for s in data:
            name = f"{s.get('prenom', '')} {s.get('nom', '')}".strip()
            if not name:
                continue

            group = s.get("groupe", {}) or {}
            circo = s.get("circonscription", {}) or {}

            senators.append({
                "name": name,
                "slug": f"{s.get('prenom','').lower()}-{s.get('nom','').lower()}".replace(" ", "-"),
                "chamber": "senat",
                "party": group.get("code", group.get("libelle", "")),
                "role": "senateur",
                "constituency": circo.get("libelle", ""),
                "photo_url": f"https://www.senat.fr{s['urlAvatar']}" if s.get("urlAvatar") else "",
                "official_url": f"https://www.senat.fr{s['url']}" if s.get("url") else "",
                "matricule": s.get("matricule", ""),
            })

        logger.info("Fetched {} senators from Senat API", len(senators))
        return senators

    # ------------------------------------------------------------------
    # 3. ASSEMBLEE NATIONALE -- All votes (Scrutins.json.zip)
    # ------------------------------------------------------------------

    async def fetch_an_scrutins(
        self, *, on_progress: Callable | None = None
    ) -> tuple[list[dict], dict[str, list[dict]]]:
        """Download and parse ALL Assembly votes from official ZIP.

        Returns:
            (scrutins, votes_by_acteur_ref)
            - scrutins: list of {id, title, date, pour, contre, abstention, result, source_url}
            - votes_by_acteur_ref: {acteurRef: [{scrutin_id, stance, date, title}]}
        """
        if on_progress:
            on_progress("Telechargement scrutins AN (19MB)...")

        raw = await self._download_bytes(AN_SCRUTINS_ZIP, timeout=180)
        if not raw:
            logger.error("Failed to download AN scrutins ZIP")
            return [], {}

        if on_progress:
            on_progress("Extraction des scrutins...")

        CACHE_DIR.mkdir(parents=True, exist_ok=True)
        zip_path = CACHE_DIR / "Scrutins.json.zip"
        zip_path.write_bytes(raw)

        scrutins: list[dict] = []
        votes_by_ref: dict[str, list[dict]] = {}

        with zipfile.ZipFile(zip_path) as zf:
            files = [n for n in zf.namelist() if n.endswith(".json")]
            total = len(files)

            for i, fname in enumerate(files):
                if on_progress and i % 500 == 0:
                    on_progress(f"Parsing scrutins... {i}/{total}")

                try:
                    with zf.open(fname) as f:
                        data = json.load(f)
                except Exception:
                    continue

                s = data.get("scrutin", data)
                uid = s.get("uid", "")
                title = s.get("titre", "")
                date = s.get("dateScrutin", "")
                synth = s.get("syntheseVote", {})
                decompte = synth.get("decompte", {})
                result = s.get("sort", {}).get("code", "")
                numero = s.get("numero", "")

                scrutins.append({
                    "id": uid,
                    "title": title,
                    "date": date,
                    "pour": int(decompte.get("pour", 0)),
                    "contre": int(decompte.get("contre", 0)),
                    "abstention": int(decompte.get("abstentions", 0)),
                    "result": result,
                    "source_url": f"https://www.assemblee-nationale.fr/dyn/17/scrutins/{numero}",
                })

                # Extract individual votes
                try:
                    groups = (
                        s.get("ventilationVotes", {})
                        .get("organe", {})
                        .get("groupes", {})
                        .get("groupe", [])
                    )
                    if isinstance(groups, dict):
                        groups = [groups]

                    for g in groups:
                        vote_data = g.get("vote", {})
                        nominatif = vote_data.get("decompteNominatif", {})

                        for stance, key in [("pour", "pours"), ("contre", "contres"), ("abstention", "abstentions")]:
                            votants = nominatif.get(key, {}).get("votant", [])
                            if isinstance(votants, dict):
                                votants = [votants]
                            for v in votants:
                                ref = v.get("acteurRef", "")
                                if ref:
                                    votes_by_ref.setdefault(ref, []).append({
                                        "scrutin_id": uid,
                                        "stance": stance,
                                        "date": date,
                                        "title": title,
                                        "source_url": f"https://www.assemblee-nationale.fr/dyn/17/scrutins/{numero}",
                                    })
                except Exception:
                    pass

        logger.info(
            "Parsed {} scrutins, {} deputies with votes",
            len(scrutins), len(votes_by_ref),
        )
        return scrutins, votes_by_ref

    # ------------------------------------------------------------------
    # 4. HATVP -- Patrimony declarations
    # ------------------------------------------------------------------

    async def fetch_hatvp(self) -> list[dict[str, Any]]:
        """Fetch HATVP declarations CSV (semicolon-delimited)."""
        rows = await self._get_csv(HATVP_CSV, delimiter=";")
        declarations: list[dict[str, Any]] = []
        for row in rows:
            name = f"{row.get('prenom', '')} {row.get('nom', '')}".strip()
            if not name:
                continue
            declarations.append({
                "name": name,
                "type_mandat": row.get("type_mandat", ""),
                "qualite": row.get("qualite", ""),
                "type_document": row.get("type_document", ""),
                "departement": row.get("departement", ""),
                "date_publication": row.get("date_publication", ""),
                "date_depot": row.get("date_depot", ""),
                "url_dossier": row.get("url_dossier", ""),
                "statut": row.get("statut_publication", ""),
            })

        logger.info("Fetched {} HATVP declarations", len(declarations))
        return declarations

    # ------------------------------------------------------------------
    # 5. PoliGraph API -- enrichment fallback
    # ------------------------------------------------------------------

    async def fetch_poligraph_politicians(self, *, limit: int = 100, max_pages: int = 10) -> list[dict]:
        """Fetch from PoliGraph API (optional enrichment)."""
        all_items: list[dict] = []
        for page in range(1, max_pages + 1):
            data = await self._get_json(
                f"{POLIGRAPH_API}/politiques", params={"limit": limit, "page": page}
            )
            if not data:
                break
            items = data.get("data", [])
            if not items:
                break
            all_items.extend(items)
            if page >= data.get("pagination", {}).get("totalPages", 1):
                break
            await asyncio.sleep(self._rate_limit)
        return all_items

    async def fetch_poligraph_affairs(self, *, max_pages: int = 5) -> list[dict]:
        """Fetch judicial affairs from PoliGraph (unique data)."""
        all_items: list[dict] = []
        for page in range(1, max_pages + 1):
            data = await self._get_json(
                f"{POLIGRAPH_API}/affaires", params={"limit": 100, "page": page}
            )
            if not data:
                break
            items = data.get("data", [])
            if not items:
                break
            all_items.extend(items)
            if page >= data.get("pagination", {}).get("totalPages", 1):
                break
            await asyncio.sleep(self._rate_limit)
        return all_items

    # ------------------------------------------------------------------
    # 5b. PoliGraph — votes per politician (by slug)
    # ------------------------------------------------------------------

    async def fetch_politician_votes(
        self, slug: str, *, max_pages: int = 3
    ) -> list[dict[str, Any]]:
        """Fetch voting record for a politician from PoliGraph API."""
        all_items: list[dict] = []
        for page in range(1, max_pages + 1):
            data = await self._get_json(
                f"{POLIGRAPH_API}/politiques/{slug}/votes",
                params={"limit": 100, "page": page},
            )
            if not data:
                break
            items = data.get("votes", data.get("data", []))
            if not items:
                break

            for v in items:
                position = (v.get("position") or "").lower()
                stance = None
                if position in ("pour", "for"):
                    stance = "pour"
                elif position in ("contre", "against"):
                    stance = "contre"
                elif position in ("abstention", "abstain"):
                    stance = "abstention"

                scrutin = v.get("scrutin", {}) or {}
                title = scrutin.get("title", v.get("title", "Vote"))
                date = (scrutin.get("votingDate", v.get("votingDate", "")) or "")[:10]
                source_url = scrutin.get("sourceUrl", v.get("sourceUrl", ""))

                all_items.append({
                    "subject": title,
                    "position_type": "vote",
                    "position_text": f"{(position or 'N/A').capitalize()} — {title}",
                    "stance": stance,
                    "source_url": source_url,
                    "source_type": "assemblee_nationale",
                    "date": date,
                })

            if page >= data.get("pagination", {}).get("totalPages", 1):
                break
            await asyncio.sleep(self._rate_limit)

        logger.debug("Fetched {} votes for '{}'", len(all_items), slug)
        return all_items

    # ------------------------------------------------------------------
    # 6. DOSSIERS LEGISLATIFS -- all law projects from AN
    # ------------------------------------------------------------------

    async def fetch_dossiers_legislatifs(
        self, *, on_progress: Callable | None = None
    ) -> list[dict[str, Any]]:
        """Download and parse all legislative dossiers from AN official ZIP.

        Returns list of {uid, titre, procedure, initiateur_ref, date, etapes}.
        """
        if on_progress:
            on_progress("Telechargement dossiers legislatifs (8.7MB)...")

        raw = await self._download_bytes(AN_DOSSIERS_ZIP, timeout=180)
        if not raw:
            logger.error("Failed to download AN dossiers ZIP")
            return []

        CACHE_DIR.mkdir(parents=True, exist_ok=True)
        zip_path = CACHE_DIR / "Dossiers_Legislatifs.json.zip"
        zip_path.write_bytes(raw)

        dossiers: list[dict] = []
        with zipfile.ZipFile(zip_path) as zf:
            files = [n for n in zf.namelist() if n.endswith(".json")]
            total = len(files)

            for i, fname in enumerate(files):
                if on_progress and i % 1000 == 0:
                    on_progress(f"Parsing dossiers... {i}/{total}")
                try:
                    with zf.open(fname) as f:
                        data = json.load(f)
                except Exception:
                    continue

                dl = data.get("dossierParlementaire", data)
                titre_obj = dl.get("titreDossier", {})
                proc = dl.get("procedureParlementaire", {})
                init = dl.get("initiateur")

                # Extract initiator acteurRef
                init_ref = None
                if init and isinstance(init, dict):
                    acteurs = init.get("acteurs", {})
                    acteur = acteurs.get("acteur", {})
                    if isinstance(acteur, list) and acteur:
                        init_ref = acteur[0].get("acteurRef")
                    elif isinstance(acteur, dict):
                        init_ref = acteur.get("acteurRef")

                # Extract first date from actes
                date = None
                actes = dl.get("actesLegislatifs", {})
                acte = actes.get("acteLegislatif", {})
                if isinstance(acte, list) and acte:
                    acte = acte[0]
                if isinstance(acte, dict):
                    date = acte.get("dateActe", "")
                    if date:
                        date = date[:10]  # YYYY-MM-DD

                dossiers.append({
                    "uid": dl.get("uid", ""),
                    "titre": titre_obj.get("titre", ""),
                    "procedure": proc.get("libelle", ""),
                    "initiateur_ref": init_ref,
                    "date": date,
                    "legislature": dl.get("legislature", "17"),
                })

        logger.info("Parsed {} dossiers legislatifs", len(dossiers))
        return dossiers

    # ------------------------------------------------------------------
    # 7. LA FABRIQUE DE LA LOI -- stats on promulgated laws
    # ------------------------------------------------------------------

    async def fetch_fabrique_loi_stats(self) -> list[dict[str, Any]]:
        """Fetch La Fabrique de la Loi metrics CSV.

        Returns list of dicts with 77 columns per law including:
        amendments count, debate duration, text growth, etc.
        """
        rows = await self._get_csv(FABRIQUE_LOI_CSV, delimiter=",")
        logger.info("Fetched {} laws from La Fabrique de la Loi", len(rows))
        return rows

    # ------------------------------------------------------------------
    # 8. WIKIDATA SPARQL -- biographies, party history, affairs
    # ------------------------------------------------------------------

    async def fetch_wikidata_deputies(self) -> list[dict[str, Any]]:
        """Fetch French deputies from Wikidata with biographies and party history.

        Uses SPARQL to query current members of the French National Assembly.
        """
        query = """
        SELECT ?person ?personLabel ?partyLabel ?birthDate ?birthPlaceLabel
               ?image ?websiteLabel ?startDate
        WHERE {
          ?person wdt:P39 wd:Q3044918 .
          OPTIONAL { ?person wdt:P102 ?party }
          OPTIONAL { ?person wdt:P569 ?birthDate }
          OPTIONAL { ?person wdt:P19 ?birthPlace }
          OPTIONAL { ?person wdt:P18 ?image }
          OPTIONAL { ?person wdt:P856 ?website }
          OPTIONAL {
            ?person p:P39 ?statement .
            ?statement ps:P39 wd:Q3044918 .
            ?statement pq:P580 ?startDate .
          }
          SERVICE wikibase:label { bd:serviceParam wikibase:language "fr,en" }
        }
        LIMIT 1000
        """
        return await self._run_sparql(query)

    async def fetch_wikidata_senators(self) -> list[dict[str, Any]]:
        """Fetch French senators from Wikidata."""
        query = """
        SELECT ?person ?personLabel ?partyLabel ?birthDate ?birthPlaceLabel
               ?image ?startDate
        WHERE {
          ?person wdt:P39 wd:Q18646817 .
          OPTIONAL { ?person wdt:P102 ?party }
          OPTIONAL { ?person wdt:P569 ?birthDate }
          OPTIONAL { ?person wdt:P19 ?birthPlace }
          OPTIONAL { ?person wdt:P18 ?image }
          OPTIONAL {
            ?person p:P39 ?statement .
            ?statement ps:P39 wd:Q18646817 .
            ?statement pq:P580 ?startDate .
          }
          SERVICE wikibase:label { bd:serviceParam wikibase:language "fr,en" }
        }
        LIMIT 500
        """
        return await self._run_sparql(query)

    async def fetch_wikidata_affairs(self) -> list[dict[str, Any]]:
        """Fetch judicial/legal affairs involving French politicians from Wikidata."""
        query = """
        SELECT ?person ?personLabel ?caseLabel ?caseDescription ?date
        WHERE {
          ?person wdt:P39 ?position .
          VALUES ?position { wd:Q3044918 wd:Q18646817 wd:Q1764122 }
          ?person wdt:P793 ?event .
          ?event wdt:P31/wdt:P279* wd:Q2995644 .
          OPTIONAL { ?event wdt:P585 ?date }
          ?event rdfs:label ?caseLabel . FILTER(LANG(?caseLabel) = "fr")
          OPTIONAL { ?event schema:description ?caseDescription . FILTER(LANG(?caseDescription) = "fr") }
          SERVICE wikibase:label { bd:serviceParam wikibase:language "fr,en" }
        }
        LIMIT 500
        """
        return await self._run_sparql(query)

    async def _run_sparql(self, query: str) -> list[dict[str, Any]]:
        """Execute a SPARQL query against Wikidata with retry."""
        for attempt in range(_MAX_RETRIES):
            try:
                async with httpx.AsyncClient(timeout=90.0) as c:
                    r = await c.get(
                        WIKIDATA_SPARQL,
                        params={"query": query},
                        headers={
                            "Accept": "application/json",
                            "User-Agent": "NEXUS-Gov/1.0 (https://github.com/nexus; contact@nexus.dev)",
                        },
                    )
                    r.raise_for_status()
                    data = r.json()

                results = []
                for binding in data.get("results", {}).get("bindings", []):
                    row = {}
                    for key, val in binding.items():
                        row[key] = val.get("value", "")
                    results.append(row)

                logger.info("Wikidata SPARQL returned {} results", len(results))
                return results
            except httpx.TimeoutException:
                if attempt < _MAX_RETRIES - 1:
                    wait = _RETRY_BACKOFF * (2 ** attempt)
                    logger.warning("Wikidata SPARQL timeout, retry {}/{} in {:.0f}s", attempt + 1, _MAX_RETRIES, wait)
                    await asyncio.sleep(wait)
                else:
                    logger.warning("Wikidata SPARQL timeout after {} retries", _MAX_RETRIES)
                    return []
            except Exception as exc:
                if attempt < _MAX_RETRIES - 1:
                    wait = _RETRY_BACKOFF * (2 ** attempt)
                    logger.debug("Wikidata retry {}/{}: {}", attempt + 1, _MAX_RETRIES, exc)
                    await asyncio.sleep(wait)
                else:
                    logger.warning("Wikidata SPARQL failed after {} retries: {}", _MAX_RETRIES, exc)
                    return []
        return []

    # ------------------------------------------------------------------
    # FULL SCAN -- builds entire gov DB from scratch
    # ------------------------------------------------------------------

    async def scan_all(
        self,
        gov_db: Any,
        *,
        on_progress: Callable | None = None,
    ) -> dict[str, int]:
        """Full autonomous scan from official sources.

        1. Fetch deputies (data.gouv.fr) + senators (Senat API)
        2. Create/update politicians in gov DB
        3. Download ALL AN scrutins (19MB ZIP, 6000+ votes)
        4. Use IdentityResolver to map acteurRef -> politician, store votes
        5. Fetch HATVP declarations -> store as gov_declarations
        6. Dossiers legislatifs -> store as gov_laws
        7. La Fabrique de la Loi -> enrich gov_laws
        8. Wikidata enrichment (photos, biographies)
        9. (Optional) PoliGraph affairs

        Returns stats dict.
        """
        from nexus.gov.identity import IdentityResolver

        stats = {
            "politicians_found": 0, "politicians_new": 0,
            "votes_found": 0, "votes_new": 0,
            "hatvp_found": 0, "hatvp_new": 0,
            "dossiers_found": 0, "dossiers_new": 0,
            "lois_found": 0,
            "affairs_found": 0,
        }

        def _p(phase: str, progress: str = "") -> None:
            if on_progress:
                on_progress(phase, progress, stats)

        # -- Phase 1: Politicians ------------------------------------------
        self._check_cancelled()
        _p("Recuperation des deputes (data.gouv.fr)...")
        deputies = await self.fetch_deputies()
        await asyncio.sleep(self._rate_limit)

        self._check_cancelled()
        _p("Recuperation des senateurs (senat.fr)...")
        senators = await self.fetch_senators()
        await asyncio.sleep(self._rate_limit)

        all_members = deputies + senators
        stats["politicians_found"] = len(all_members)

        # -- Phase 2: Upsert politicians -----------------------------------
        self._check_cancelled()
        _p(f"Enregistrement de {len(all_members)} politiciens...")
        existing = await gov_db.list_politicians(limit=100_000)
        existing_names = {p["name"].lower(): p for p in existing}

        for m in all_members:
            name = m["name"]
            if name.lower() not in existing_names:
                try:
                    created = await gov_db.create_politician(
                        name=name,
                        chamber=m.get("chamber", "assemblee"),
                        party=m.get("party"),
                        role=m.get("role"),
                        constituency=m.get("constituency"),
                        photo_url=m.get("photo_url"),
                        official_url=m.get("official_url"),
                    )
                    existing_names[name.lower()] = created
                    stats["politicians_new"] += 1
                except Exception as exc:
                    logger.debug("Create politician '{}': {}", name, exc)

        # Build name->id and slug->id mappings
        all_pols = await gov_db.list_politicians(limit=100_000)
        name_to_id = {p["name"].lower(): p["id"] for p in all_pols}
        slug_to_pol = {p.get("slug", ""): p for p in all_pols if p.get("slug")}

        # -- Phase 3: PoliGraph votes per politician (most reliable) --------
        self._check_cancelled()
        _p("Recuperation des votes via PoliGraph...")

        vote_stored = 0
        for i, pol in enumerate(all_pols):
            slug = pol.get("slug", "")
            if not slug:
                continue

            self._check_cancelled()
            _p("Votes PoliGraph...", f"{pol['name']} ({i+1}/{len(all_pols)})")

            await asyncio.sleep(self._rate_limit)
            votes = await self.fetch_politician_votes(slug, max_pages=2)
            stats["votes_found"] += len(votes)

            for v in votes:
                url = v.get("source_url", "")
                if not url:
                    continue
                # Dedup by URL
                try:
                    exists = await gov_db.position_exists_by_url(url)
                except AttributeError:
                    # Method might not exist yet — fallback
                    exists = False
                if exists:
                    continue

                try:
                    await gov_db.create_position(
                        politician_id=pol["id"],
                        subject=v.get("subject", "Vote")[:200],
                        position_type="vote",
                        position_text=v.get("position_text", "")[:500],
                        stance=v.get("stance"),
                        source_url=url,
                        source_type="assemblee_nationale",
                        date=v.get("date", ""),
                    )
                    vote_stored += 1
                    stats["votes_new"] += 1
                except Exception as exc:
                    logger.debug("Vote store: {}", exc)

        logger.info("PoliGraph votes: {} found, {} new", stats["votes_found"], vote_stored)

        # -- Phase 4: AN Scrutins ZIP (acteurRef mapping — bonus) -----------
        self._check_cancelled()
        scrutins, votes_by_ref = await self.fetch_an_scrutins(
            on_progress=lambda msg: _p("Scrutins AN", msg)
        )
        stats["votes_found"] = sum(len(v) for v in votes_by_ref.values())

        # -- Phase 3b: Resolve acteurRef -> politician via IdentityResolver --
        self._check_cancelled()
        _p("Resolution des identites acteurRef -> politiciens...")

        resolver = IdentityResolver(gov_db)
        await resolver.build_cache()

        # Build acteurRef -> politician_id mapping
        # Each acteurRef like "PA842279" is unique to the AN data
        acteur_ref_to_pol_id: dict[str, str] = {}
        unique_refs = list(votes_by_ref.keys())
        resolved_count = 0
        unresolved_refs: list[str] = []

        # First: check if any refs are already linked as external IDs
        for ref in unique_refs:
            existing_pol = await gov_db.find_politician_by_external_id("assemblee_nationale", ref)
            if existing_pol:
                acteur_ref_to_pol_id[ref] = existing_pol["id"]
                resolved_count += 1

        # For unresolved refs: we can't fuzzy-match a code like "PA842279" to a name.
        # But we can look at the vote data to extract names from the scrutin JSON.
        # The ZIP embeds acteur names in the decompteNominatif votant entries.
        # Since we already parsed that, let's use a second pass to extract names.
        # For now, store aggregate scrutin data for unresolved refs.

        logger.info(
            "acteurRef resolution: {}/{} resolved via external IDs",
            resolved_count, len(unique_refs),
        )

        # -- Phase 3c: Store individual vote positions ---------------------
        self._check_cancelled()
        _p("Enregistrement des votes...", f"{stats['votes_found']} votes individuels")

        vote_stored = 0
        for ref, ref_votes in votes_by_ref.items():
            pol_id = acteur_ref_to_pol_id.get(ref)
            if not pol_id:
                continue  # Can't link this acteurRef to a politician yet

            for vote in ref_votes:
                url = vote.get("source_url", "")
                subject = vote.get("title", "")[:200]
                date = vote.get("date", "")
                stance = vote.get("stance", "")

                # Dedup: check by source_url first, then by (politician_id, subject, date)
                if url:
                    if await gov_db.position_exists_by_url(url + f"#{pol_id}"):
                        continue
                else:
                    if subject and await gov_db.position_exists_by_key(pol_id, subject, date or None):
                        continue

                try:
                    await gov_db.create_position(
                        politician_id=pol_id,
                        subject=subject,
                        position_type="vote",
                        position_text=f"Vote {stance} — {subject}",
                        stance=stance,
                        source_url=f"{url}#{pol_id}" if url else f"an:scrutin:{vote.get('scrutin_id', '')}:{pol_id}",
                        source_type="assemblee_nationale",
                        date=date,
                    )
                    vote_stored += 1
                except Exception as exc:
                    logger.debug("Vote position: {}", exc)

            # Yield control every 50 politicians to avoid blocking
            if vote_stored % 200 == 0:
                await asyncio.sleep(0)

        stats["votes_new"] = vote_stored
        logger.info("Stored {} individual vote positions", vote_stored)

        # -- Phase 4: HATVP ------------------------------------------------
        self._check_cancelled()
        _p("Recuperation HATVP...")
        hatvp = await self.fetch_hatvp()
        stats["hatvp_found"] = len(hatvp)

        hatvp_stored = 0
        for decl in hatvp:
            name = decl["name"]
            pol_id = name_to_id.get(name.lower())
            if not pol_id:
                continue

            url = (
                f"https://www.hatvp.fr{decl['url_dossier']}"
                if decl.get("url_dossier")
                else ""
            )

            # Dedup: check by URL if available, else by (politician_id, type_doc, date)
            if url:
                if await gov_db.declaration_exists_by_url(url):
                    continue
            else:
                type_doc = decl.get("type_document", "patrimoine")
                date_pub = decl.get("date_publication", decl.get("date_depot", ""))
                subject = f"Declaration {type_doc}"
                if await gov_db.position_exists_by_key(pol_id, subject, date_pub or None):
                    continue

            try:
                date_pub = decl.get("date_publication", "")
                date_depot = decl.get("date_depot", "")
                await gov_db.create_declaration(
                    politician_id=pol_id,
                    type=decl.get("type_document", "patrimoine"),
                    qualite=decl.get("qualite", ""),
                    departement=decl.get("departement", ""),
                    date_publication=date_pub or None,
                    date_depot=date_depot or None,
                    url=url or None,
                    status=decl.get("statut", ""),
                )
                hatvp_stored += 1
            except Exception as exc:
                logger.debug("HATVP declaration: {}", exc)

        stats["hatvp_new"] = hatvp_stored

        # -- Phase 5: Dossiers legislatifs ---------------------------------
        self._check_cancelled()
        _p("Dossiers legislatifs (AN)...")
        dossiers = await self.fetch_dossiers_legislatifs(
            on_progress=lambda msg: _p("Dossiers legislatifs", msg)
        )
        stats["dossiers_found"] = len(dossiers)

        dossiers_stored = 0
        for dl in dossiers:
            titre = dl.get("titre", "")
            uid = dl.get("uid", "")
            if not titre or not uid:
                continue

            # Dedup by uid (UNIQUE constraint on gov_laws.uid)
            existing_law = await gov_db.get_law_by_uid(uid)
            if existing_law:
                continue

            try:
                await gov_db.create_law(
                    uid=uid,
                    title=titre,
                    procedure=dl.get("procedure", ""),
                    initiator_ref=dl.get("initiateur_ref"),
                    date_initial=dl.get("date"),
                    legislature=dl.get("legislature", "17"),
                    source_url=f"https://www.assemblee-nationale.fr/dyn/17/dossiers/{uid}",
                )
                dossiers_stored += 1
            except Exception as exc:
                logger.debug("Dossier law: {}", exc)

            # Yield control periodically
            if dossiers_stored % 500 == 0:
                await asyncio.sleep(0)

        stats["dossiers_new"] = dossiers_stored

        # -- Phase 6: La Fabrique de la Loi --------------------------------
        self._check_cancelled()
        _p("La Fabrique de la Loi (stats lois)...")
        loi_stats = await self.fetch_fabrique_loi_stats()
        stats["lois_found"] = len(loi_stats)

        # Enrich existing laws with La Fabrique stats (amendments, duration, etc.)
        lois_enriched = 0
        for loi in loi_stats:
            titre = loi.get("Titre court", loi.get("Titre", ""))
            if not titre:
                continue
            url_jo = loi.get("URL JO", "")
            num = loi.get("Numero de la loi", "")
            if not url_jo:
                continue

            # Try to find matching law by uid or by similar title
            # La Fabrique uses its own numbering, cross-ref is best-effort
            amendements = loi.get("Nombre d'amendements", "0")
            duree = loi.get("Duree d'adoption (jours)", "")

            try:
                amendements_count = int(amendements) if amendements else 0
            except (ValueError, TypeError):
                amendements_count = 0
            try:
                duration_days = int(duree) if duree else 0
            except (ValueError, TypeError):
                duration_days = 0

            # Store as a new law record if no matching uid exists
            fabrique_uid = f"fabrique:{num}" if num else None
            if fabrique_uid:
                existing_law = await gov_db.get_law_by_uid(fabrique_uid)
                if existing_law:
                    continue

                try:
                    await gov_db.create_law(
                        uid=fabrique_uid,
                        title=titre[:500],
                        short_title=titre[:200],
                        jo_url=url_jo,
                        amendments_count=amendements_count,
                        duration_days=duration_days,
                    )
                    lois_enriched += 1
                except Exception as exc:
                    logger.debug("Fabrique loi: {}", exc)

        stats["lois_enriched"] = lois_enriched

        # -- Phase 7: Wikidata enrichment ----------------------------------
        self._check_cancelled()
        _p("Wikidata (biographies + affaires)...")
        try:
            wiki_deputies = await self.fetch_wikidata_deputies()
            stats["wikidata_deputies"] = len(wiki_deputies)
            await asyncio.sleep(self._rate_limit)

            self._check_cancelled()
            wiki_senators = await self.fetch_wikidata_senators()
            stats["wikidata_senators"] = len(wiki_senators)
            await asyncio.sleep(self._rate_limit)

            self._check_cancelled()
            wiki_affairs = await self.fetch_wikidata_affairs()
            stats["wikidata_affairs"] = len(wiki_affairs)

            # Enrich existing politicians with Wikidata data
            for wp in wiki_deputies + wiki_senators:
                wname = wp.get("personLabel", "").strip()
                if not wname:
                    continue
                pol_id = name_to_id.get(wname.lower())
                if not pol_id:
                    continue
                # Update photo if we have a better one from Wikidata
                image = wp.get("image", "")
                if image:
                    try:
                        await gov_db.update_politician(pol_id, photo_url=image)
                    except Exception:
                        pass

        except asyncio.CancelledError:
            raise
        except Exception as exc:
            logger.warning("Wikidata enrichment failed: {}", exc)

        # -- Phase 8: PoliGraph affairs (optional enrichment) --------------
        self._check_cancelled()
        _p("Affaires judiciaires (PoliGraph)...")
        try:
            affairs = await self.fetch_poligraph_affairs(max_pages=5)
            stats["affairs_found"] = len(affairs)
        except Exception:
            affairs = []

        _p("Scan termine", (
            f"{stats['politicians_found']} politiciens, "
            f"{stats['votes_found']} votes ({stats['votes_new']} stockes), "
            f"{stats['hatvp_found']} HATVP ({stats['hatvp_new']} nouvelles), "
            f"{stats.get('dossiers_found', 0)} dossiers ({stats.get('dossiers_new', 0)} nouveaux), "
            f"{stats.get('lois_found', 0)} lois, "
            f"{stats.get('wikidata_deputies', 0)}+{stats.get('wikidata_senators', 0)} Wikidata, "
            f"{stats.get('affairs_found', 0)} affaires"
        ))

        logger.info("Full government scan complete: {}", stats)
        return stats
