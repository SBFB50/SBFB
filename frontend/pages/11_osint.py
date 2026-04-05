import sys; from pathlib import Path; sys.path.insert(0, str(Path(__file__).resolve().parent.parent)) if str(Path(__file__).resolve().parent.parent) not in sys.path else None
"""
NEXUS -- OSINT Reconnaissance page.

Passive recon tools: email checks (holehe + social), username
search across platforms, domain WHOIS/DNS, and automated
entity scanning.
"""

import streamlit as st
from frontend.api_client import api
from frontend.components.system_stats import render_system_stats; render_system_stats()

st.header("OSINT - Reconnaissance")

case_id = st.session_state.get("case_id")

# ====================================================================
# Tabs
# ====================================================================

tab_email, tab_username, tab_domain, tab_auto = st.tabs([
    "Email",
    "Username",
    "Domaine",
    "Auto-scan",
])


# ====================================================================
# Tab 1 -- Email
# ====================================================================

with tab_email:
    st.subheader("Recherche par email")
    st.caption(
        "Verifie si un email est enregistre sur 120+ services (holehe) "
        "et recherche les profils sociaux associes."
    )

    email_input = st.text_input(
        "Adresse email",
        placeholder="cible@example.com",
        key="osint_email_input",
    )

    if st.button("Scanner", key="scan_email", type="primary"):
        if not email_input.strip():
            st.warning("Entrez une adresse email.")
        else:
            with st.spinner("Scan en cours (holehe + profils sociaux)..."):
                result = api.recon_email(email_input.strip())

            if result:
                # -- Holehe results --
                holehe = result.get("holehe", [])
                st.markdown(
                    f"#### Holehe -- {result.get('holehe_count', 0)} "
                    f"site(s) trouve(s)"
                )
                if holehe:
                    for h in holehe:
                        st.markdown(
                            f"- **{h.get('site', '?')}** ({h.get('domain', '')})"
                        )
                else:
                    st.info("Aucun site trouve par holehe pour cet email.")

                # -- Social results --
                social = result.get("social", [])
                found_social = [s for s in social if s.get("exists")]
                st.markdown(
                    f"#### Profils sociaux -- "
                    f"{result.get('social_found', 0)} trouve(s)"
                )
                if found_social:
                    for s in found_social:
                        st.markdown(
                            f"- **{s.get('platform', '?')}** -- "
                            f"[{s.get('url', '')}]({s.get('url', '')})"
                        )
                else:
                    st.info("Aucun profil social trouve.")

                # -- Not found --
                not_found = [s for s in social if not s.get("exists")]
                if not_found:
                    with st.expander(
                        f"Plateformes sans resultat ({len(not_found)})"
                    ):
                        for s in not_found:
                            status = s.get("status_code", 0)
                            st.caption(
                                f"{s.get('platform', '?')} -- "
                                f"status {status}"
                            )


# ====================================================================
# Tab 2 -- Username
# ====================================================================

with tab_username:
    st.subheader("Recherche par username")
    st.caption(
        "Recherche un pseudo sur les principales plateformes sociales."
    )

    username_input = st.text_input(
        "Username",
        placeholder="johndoe42",
        key="osint_username_input",
    )

    if st.button("Scanner", key="scan_username", type="primary"):
        if not username_input.strip():
            st.warning("Entrez un username.")
        else:
            with st.spinner("Scan des plateformes sociales..."):
                result = api.recon_username(username_input.strip())

            if result:
                found = result.get("found_count", 0)
                results_list = result.get("results", [])

                st.markdown(f"#### {found} profil(s) trouve(s)")

                found_profiles = [r for r in results_list if r.get("exists")]
                if found_profiles:
                    for p in found_profiles:
                        st.markdown(
                            f"- **{p.get('platform', '?')}** -- "
                            f"[{p.get('url', '')}]({p.get('url', '')})"
                        )

                not_found = [r for r in results_list if not r.get("exists")]
                if not_found:
                    with st.expander(
                        f"Plateformes sans resultat ({len(not_found)})"
                    ):
                        for p in not_found:
                            status = p.get("status_code", 0)
                            st.caption(
                                f"{p.get('platform', '?')} -- "
                                f"status {status}"
                            )


# ====================================================================
# Tab 3 -- Domain
# ====================================================================

with tab_domain:
    st.subheader("Recherche par domaine")
    st.caption("WHOIS + resolution DNS (A, MX, NS).")

    domain_input = st.text_input(
        "Nom de domaine",
        placeholder="example.com",
        key="osint_domain_input",
    )

    if st.button("Scanner", key="scan_domain", type="primary"):
        if not domain_input.strip():
            st.warning("Entrez un nom de domaine.")
        else:
            with st.spinner("WHOIS + DNS en cours..."):
                result = api.recon_domain(domain_input.strip())

            if result:
                # -- WHOIS --
                whois_data = result.get("whois", {})
                st.markdown("#### WHOIS")

                if whois_data.get("error"):
                    st.error(f"Erreur WHOIS: {whois_data['error']}")
                else:
                    col1, col2 = st.columns(2)
                    with col1:
                        st.markdown(
                            f"**Registrar:** {whois_data.get('registrar', 'N/A')}"
                        )
                        st.markdown(
                            f"**Creation:** {whois_data.get('creation_date', 'N/A')}"
                        )
                        st.markdown(
                            f"**Expiration:** "
                            f"{whois_data.get('expiration_date', 'N/A')}"
                        )
                    with col2:
                        st.markdown(
                            f"**Nom:** {whois_data.get('registrant_name', 'N/A')}"
                        )
                        st.markdown(
                            f"**Email:** {whois_data.get('registrant_email', 'N/A')}"
                        )

                    ns_list = whois_data.get("name_servers", [])
                    if ns_list:
                        st.markdown("**Name servers:**")
                        for ns in ns_list:
                            st.markdown(f"- `{ns}`")

                # -- DNS --
                dns_data = result.get("dns", {})
                st.markdown("#### DNS")

                a_records = dns_data.get("a_records", [])
                mx_records = dns_data.get("mx_records", [])
                ns_records = dns_data.get("ns_records", [])

                col1, col2, col3 = st.columns(3)
                with col1:
                    st.markdown("**A Records**")
                    if a_records:
                        for r in a_records:
                            st.code(r)
                    else:
                        st.caption("Aucun")
                with col2:
                    st.markdown("**MX Records**")
                    if mx_records:
                        for r in mx_records:
                            st.code(r)
                    else:
                        st.caption("Aucun")
                with col3:
                    st.markdown("**NS Records**")
                    if ns_records:
                        for r in ns_records:
                            st.code(r)
                    else:
                        st.caption("Aucun")


# ====================================================================
# Tab 4 -- Auto-scan
# ====================================================================

with tab_auto:
    st.subheader("Scan automatique des entites")
    st.caption(
        "Lance la reconnaissance OSINT sur toutes les entites "
        "de type email et account du dossier actif."
    )

    if not case_id:
        st.warning("Selectionnez un dossier dans la barre laterale.")
    else:
        # Show existing recon results
        recon_entities = api.get_case_recon(case_id)
        if recon_entities:
            st.markdown(
                f"**{len(recon_entities)} entite(s) avec resultats de recon**"
            )
            for ent in recon_entities:
                meta = ent.get("metadata", {})
                recon = meta.get("recon", {}) if isinstance(meta, dict) else {}
                with st.expander(
                    f"{ent.get('name', '?')} ({ent.get('entity_type', '?')})"
                ):
                    holehe_count = recon.get("holehe_count", 0)
                    social_found = recon.get("social_found", 0)
                    st.markdown(
                        f"- Holehe: **{holehe_count}** site(s)\n"
                        f"- Social: **{social_found}** profil(s)"
                    )

                    holehe_list = recon.get("holehe", [])
                    if holehe_list:
                        st.markdown("**Sites holehe:**")
                        for h in holehe_list:
                            st.caption(
                                f"  {h.get('site', '?')} ({h.get('domain', '')})"
                            )

                    social_list = recon.get("social", [])
                    found_social = [s for s in social_list if s.get("exists")]
                    if found_social:
                        st.markdown("**Profils sociaux:**")
                        for s in found_social:
                            st.caption(
                                f"  {s.get('platform', '?')} -- {s.get('url', '')}"
                            )

        st.markdown("---")

        if st.button(
            "Scanner toutes les entites du dossier",
            key="auto_recon",
            type="primary",
        ):
            with st.spinner(
                "Scan OSINT en cours sur toutes les entites email/account..."
            ):
                result = api.recon_auto(case_id)

            if result:
                scanned = result.get("scanned", 0)
                errors = result.get("errors", 0)

                if scanned > 0:
                    st.success(
                        f"Scan termine: {scanned} entite(s) scannee(s), "
                        f"{errors} erreur(s)."
                    )
                else:
                    st.info(
                        "Aucune entite email/account trouvee dans ce dossier."
                    )

                # Show results
                for item in result.get("results", []):
                    recon = item.get("recon", {})
                    with st.expander(
                        f"{item.get('name', '?')} ({item.get('type', '?')})"
                    ):
                        st.json(recon)

                st.rerun()
