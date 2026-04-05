"""
NEXUS -- Investigation Map page.

Interactive folium map showing geocoded locations for the active case,
with route calculation and travel-time verification.
"""

from __future__ import annotations
import sys; from pathlib import Path; sys.path.insert(0, str(Path(__file__).resolve().parent.parent)) if str(Path(__file__).resolve().parent.parent) not in sys.path else None  # noqa: E402

import folium
import streamlit as st
from streamlit_folium import st_folium

from frontend.api_client import api
from frontend.components.system_stats import render_system_stats; render_system_stats()

# ---------------------------------------------------------------------------
# Colour / icon mapping per location_type
# ---------------------------------------------------------------------------

_LOCATION_COLOURS = {
    "crime_scene": "red",
    "home": "blue",
    "work": "green",
    "hospital": "orange",
    "establishment": "purple",
    "other": "gray",
}

_LOCATION_ICONS = {
    "crime_scene": "exclamation-sign",
    "home": "home",
    "work": "briefcase",
    "hospital": "plus-sign",
    "establishment": "glass",
    "other": "map-marker",
}

_LOCATION_LABELS = {
    "crime_scene": "Scene de crime",
    "home": "Domicile",
    "work": "Lieu de travail",
    "hospital": "Hopital",
    "establishment": "Etablissement",
    "other": "Autre",
}

# ---------------------------------------------------------------------------
# Guard
# ---------------------------------------------------------------------------

st.title("Carte d'investigation")

case_id = st.session_state.get("case_id")
if not case_id:
    st.info("Selectionnez ou creez un dossier dans la barre laterale.")
    st.stop()

# ---------------------------------------------------------------------------
# 1. Geocoding
# ---------------------------------------------------------------------------

st.subheader("Geocodage des lieux")

col_btn, col_status = st.columns([1, 3])

with col_btn:
    do_geocode = st.button("Geocoder les lieux", type="primary")

if do_geocode:
    with st.spinner("Geocodage en cours (1 lieu/sec, API gratuite)..."):
        result = api.geocode_case(case_id)
    if result:
        geo = result.get("geocoded", 0)
        cached = result.get("cached", 0)
        nf = result.get("not_found", 0)
        st.success(
            f"Termine : {geo} geocode(s), {cached} en cache, {nf} non trouve(s)."
        )
        if result.get("results"):
            with st.expander("Details du geocodage"):
                for r in result["results"]:
                    icon = "✅" if r["status"] in ("geocoded", "cached") else "❌"
                    st.write(f"{icon} **{r['name']}** — {r['status']}")
    else:
        st.warning("Aucune entite de type 'location' a geocoder.")

# ---------------------------------------------------------------------------
# 2. Map
# ---------------------------------------------------------------------------

st.subheader("Carte")

map_data = api.get_case_map(case_id)

if not map_data or not map_data.get("locations"):
    st.caption(
        "Aucun lieu geocode pour ce dossier. "
        "Ajoutez des entites de type 'location' puis cliquez sur 'Geocoder les lieux'."
    )
    st.stop()

locations = [
    loc for loc in map_data["locations"]
    if loc.get("lat") is not None and loc.get("lon") is not None
]

if not locations:
    st.caption("Aucun lieu avec coordonnees GPS. Relancez le geocodage.")
    st.stop()

# Build entity lookup for popups
entity_lookup: dict[str, list[dict]] = {}
for ea in map_data.get("entities_at_locations", []):
    lid = ea["location_id"]
    entity_lookup.setdefault(lid, []).append(ea)

# Legend
st.markdown("**Legende :** " + " | ".join(
    f":{_LOCATION_COLOURS.get(lt, 'gray')}_circle: {_LOCATION_LABELS.get(lt, lt)}"
    for lt in sorted(_LOCATION_LABELS)
))

# Centre the map on the mean of all points
mean_lat = sum(l["lat"] for l in locations) / len(locations)
mean_lon = sum(l["lon"] for l in locations) / len(locations)

m = folium.Map(location=[mean_lat, mean_lon], zoom_start=12)

for loc in locations:
    loc_type = loc.get("location_type", "other")
    colour = _LOCATION_COLOURS.get(loc_type, "gray")
    icon_name = _LOCATION_ICONS.get(loc_type, "map-marker")
    label = _LOCATION_LABELS.get(loc_type, "Autre")

    # Build popup HTML
    popup_lines = [
        f"<b>{loc['name']}</b>",
        f"<i>{label}</i>",
    ]
    if loc.get("address"):
        popup_lines.append(f"<small>{loc['address']}</small>")

    entities_here = entity_lookup.get(loc["id"], [])
    if entities_here:
        popup_lines.append("<hr style='margin:4px 0'>")
        for ea in entities_here:
            popup_lines.append(
                f"<small>Entite: {ea['entity_name']} ({ea['entity_type']})</small>"
            )

    popup_html = "<br>".join(popup_lines)

    folium.Marker(
        location=[loc["lat"], loc["lon"]],
        popup=folium.Popup(popup_html, max_width=300),
        tooltip=loc["name"],
        icon=folium.Icon(color=colour, icon=icon_name, prefix="glyphicon"),
    ).add_to(m)

st_folium(m, width=900, height=550, returned_objects=[])

# ---------------------------------------------------------------------------
# 3. Travel-time verification
# ---------------------------------------------------------------------------

st.markdown("---")
st.subheader("Verification de trajet")

loc_names = [loc["name"] for loc in locations]

if len(loc_names) < 2:
    st.caption("Il faut au moins 2 lieux geocodes pour verifier un trajet.")
else:
    col1, col2 = st.columns(2)
    with col1:
        origin = st.selectbox("Origine", options=loc_names, key="route_origin")
    with col2:
        remaining = [n for n in loc_names if n != origin]
        destination = st.selectbox(
            "Destination",
            options=remaining if remaining else loc_names,
            key="route_dest",
        )

    claimed = st.number_input(
        "Temps declare (minutes)", min_value=0.0, value=30.0, step=5.0
    )

    if st.button("Verifier le trajet"):
        with st.spinner("Calcul du trajet via OSRM..."):
            result = api.verify_travel(case_id, origin, destination, claimed)

        if result and "error" not in result:
            plausible = result.get("plausible", False)
            actual = result.get("actual_minutes", "?")
            diff = result.get("difference_minutes", "?")
            dist = result.get("distance_km", "?")

            if plausible:
                st.success(
                    f"PLAUSIBLE -- Temps estime : {actual} min | "
                    f"Distance : {dist} km | Ecart : {diff:+} min"
                )
            else:
                st.error(
                    f"SUSPECT -- Temps estime : {actual} min | "
                    f"Distance : {dist} km | Ecart : {diff:+} min"
                )

            st.markdown(
                f"- **Declare** : {claimed} min\n"
                f"- **Estime (conduite)** : {actual} min\n"
                f"- **Distance** : {dist} km\n"
                f"- **Ecart** : {diff:+} min"
            )
        elif result and "error" in result:
            st.warning(result["error"])
        else:
            st.error("Erreur lors du calcul du trajet.")

# ---------------------------------------------------------------------------
# 4. Route calculation (standalone)
# ---------------------------------------------------------------------------

st.markdown("---")
st.subheader("Calculer un trajet")

if len(loc_names) >= 2:
    col1, col2 = st.columns(2)
    with col1:
        r_origin = st.selectbox(
            "Origine", options=loc_names, key="calc_origin"
        )
    with col2:
        r_remaining = [n for n in loc_names if n != r_origin]
        r_destination = st.selectbox(
            "Destination",
            options=r_remaining if r_remaining else loc_names,
            key="calc_dest",
        )

    if st.button("Calculer", key="calc_route_btn"):
        with st.spinner("Calcul..."):
            route = api.calculate_route(case_id, r_origin, r_destination)

        if route and "error" not in route:
            st.success(
                f"{r_origin} -> {r_destination} : "
                f"{route['distance_km']} km, {route['duration_min']} min"
            )
        elif route and "error" in route:
            st.warning(route["error"])
        else:
            st.error("Erreur lors du calcul.")

# ---------------------------------------------------------------------------
# 5. Location table
# ---------------------------------------------------------------------------

st.markdown("---")
st.subheader("Tous les lieux geocodes")

import pandas as pd

df = pd.DataFrame([
    {
        "Nom": loc["name"],
        "Type": _LOCATION_LABELS.get(loc.get("location_type", "other"), "Autre"),
        "Adresse": loc.get("address", ""),
        "Lat": loc.get("lat"),
        "Lon": loc.get("lon"),
    }
    for loc in locations
])

st.dataframe(df, use_container_width=True, hide_index=True)
