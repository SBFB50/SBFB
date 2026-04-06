"""
NEXUS -- Timeline export (HTML standalone + PNG image).

Produces:
- Standalone HTML timeline using embedded CSS/JS
- PNG image via Plotly (if installed)
"""

from __future__ import annotations

import json
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, List, Optional

from loguru import logger


class TimelineExporter:
    """Export case timelines to standalone HTML or PNG images.

    Usage::

        exporter = TimelineExporter()
        exporter.export_to_html(events, Path("timeline.html"), title="Case X")
        exporter.export_to_image(events, Path("timeline.png"), title="Case X")
    """

    # ------------------------------------------------------------------
    # HTML export (standalone, no dependencies)
    # ------------------------------------------------------------------

    def export_to_html(
        self,
        events: List[Dict[str, Any]],
        output_path: Path,
        *,
        title: str = "Timeline",
        case_info: Optional[Dict[str, Any]] = None,
    ) -> Path:
        """Export timeline events to a standalone HTML file.

        The file includes all CSS/JS inline — no external dependencies.

        Parameters
        ----------
        events : list[dict]
            Each dict must have at least ``date`` and ``title``.
            Optional: ``type``, ``description``, ``related_id``.
        output_path : Path
            Where to write the HTML file.
        title : str
            Title shown in the page header.
        case_info : dict, optional
            Case metadata to display in the header.

        Returns
        -------
        Path
            The output path.
        """
        output_path.parent.mkdir(parents=True, exist_ok=True)

        events_json = json.dumps(events, ensure_ascii=False, default=str)
        case_name = (case_info or {}).get("name", "")
        case_ref = (case_info or {}).get("reference", "")
        generated_at = datetime.now(timezone.utc).strftime("%d/%m/%Y %H:%M UTC")

        html = _TIMELINE_HTML_TEMPLATE.format(
            title=_escape_html(title),
            case_name=_escape_html(case_name),
            case_ref=_escape_html(case_ref),
            generated_at=generated_at,
            events_json=events_json,
            event_count=len(events),
        )

        output_path.write_text(html, encoding="utf-8")
        logger.info(
            "Timeline HTML exported: {} ({} events)",
            output_path,
            len(events),
        )
        return output_path

    # ------------------------------------------------------------------
    # PNG export (via Plotly)
    # ------------------------------------------------------------------

    def export_to_image(
        self,
        events: List[Dict[str, Any]],
        output_path: Path,
        *,
        title: str = "Timeline",
        width: int = 1400,
        height: int = 600,
    ) -> Path:
        """Export timeline events to a PNG image using Plotly.

        Requires ``plotly`` and ``kaleido`` packages.

        Parameters
        ----------
        events : list[dict]
            Timeline event dicts with ``date`` and ``title``.
        output_path : Path
            Where to write the PNG file.
        title : str
            Chart title.
        width, height : int
            Image dimensions in pixels.

        Returns
        -------
        Path
            The output path.
        """
        try:
            import plotly.graph_objects as go
        except ImportError:
            raise ImportError(
                "Plotly is required for image export. "
                "Install with: pip install plotly kaleido"
            )

        output_path.parent.mkdir(parents=True, exist_ok=True)

        # Filter events with valid dates
        dated = [e for e in events if e.get("date")]
        if not dated:
            logger.warning("No dated events to plot")
            # Create an empty chart
            fig = go.Figure()
            fig.update_layout(
                title=title,
                annotations=[{
                    "text": "Aucun evenement date",
                    "xref": "paper",
                    "yref": "paper",
                    "showarrow": False,
                    "font": {"size": 16},
                }],
            )
            fig.write_image(str(output_path), width=width, height=height)
            return output_path

        # Parse dates
        dates = []
        labels = []
        colors = []
        hover_texts = []

        color_map = {
            "evidence": "#1976D2",
            "entity": "#388E3C",
            "hypothesis_update": "#F57C00",
            "hypothesis_snapshot": "#F57C00",
            "monitoring_result": "#7B1FA2",
            "event": "#C62828",
        }

        for ev in dated:
            try:
                d = ev["date"]
                if isinstance(d, str):
                    d = datetime.fromisoformat(d.replace("Z", "+00:00"))
                dates.append(d)
            except (ValueError, TypeError):
                continue

            labels.append(ev.get("title", "?"))
            event_type = ev.get("type", "other")
            colors.append(color_map.get(event_type, "#757575"))
            hover_texts.append(
                f"<b>{ev.get('title', '')}</b><br>"
                f"Type: {event_type}<br>"
                f"Date: {ev.get('date', '')}<br>"
                f"{ev.get('description', '')}"
            )

        # Build Y positions (alternating above/below the line)
        y_positions = [((i % 2) * 2 - 1) * (1 + (i % 3) * 0.5) for i in range(len(dates))]

        fig = go.Figure()

        # Main scatter for events
        fig.add_trace(go.Scatter(
            x=dates,
            y=y_positions,
            mode="markers+text",
            marker=dict(size=12, color=colors, line=dict(width=1, color="white")),
            text=labels,
            textposition=["top center" if y > 0 else "bottom center" for y in y_positions],
            textfont=dict(size=9),
            hovertext=hover_texts,
            hoverinfo="text",
            showlegend=False,
        ))

        # Horizontal axis line
        fig.add_hline(y=0, line_dash="solid", line_color="#BDBDBD", line_width=2)

        # Vertical lines connecting dots to axis
        for i, (d, y) in enumerate(zip(dates, y_positions)):
            fig.add_shape(
                type="line",
                x0=d, y0=0, x1=d, y1=y,
                line=dict(color=colors[i], width=1, dash="dot"),
            )

        fig.update_layout(
            title=dict(text=title, font=dict(size=18)),
            xaxis=dict(title="Date", showgrid=True, gridcolor="#E0E0E0"),
            yaxis=dict(visible=False, range=[-4, 4]),
            plot_bgcolor="white",
            paper_bgcolor="white",
            margin=dict(l=40, r=40, t=60, b=40),
            font=dict(family="Arial, sans-serif"),
        )

        fig.write_image(str(output_path), width=width, height=height)
        logger.info(
            "Timeline image exported: {} ({} events)",
            output_path,
            len(dated),
        )
        return output_path


# ------------------------------------------------------------------
# Helpers
# ------------------------------------------------------------------

def _escape_html(text: str) -> str:
    """Basic HTML escape."""
    return (
        text.replace("&", "&amp;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
        .replace('"', "&quot;")
    )


# ------------------------------------------------------------------
# Standalone HTML template
# ------------------------------------------------------------------

_TIMELINE_HTML_TEMPLATE = """<!DOCTYPE html>
<html lang="fr">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>{title} — NEXUS</title>
<style>
  * {{ margin: 0; padding: 0; box-sizing: border-box; }}
  body {{
    font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
    background: #0a0e17;
    color: #e0e0e0;
    padding: 2rem;
  }}
  .header {{
    text-align: center;
    margin-bottom: 2rem;
    padding-bottom: 1rem;
    border-bottom: 2px solid #1a2332;
  }}
  .header h1 {{
    font-size: 2rem;
    color: #4fc3f7;
    margin-bottom: 0.5rem;
  }}
  .header .meta {{
    color: #78909c;
    font-size: 0.9rem;
  }}
  .stats {{
    display: flex;
    justify-content: center;
    gap: 2rem;
    margin-bottom: 2rem;
  }}
  .stat-box {{
    background: #1a2332;
    padding: 1rem 2rem;
    border-radius: 8px;
    text-align: center;
  }}
  .stat-box .number {{
    font-size: 2rem;
    font-weight: bold;
    color: #4fc3f7;
  }}
  .stat-box .label {{
    color: #78909c;
    font-size: 0.85rem;
  }}
  .timeline {{
    position: relative;
    max-width: 900px;
    margin: 0 auto;
    padding: 1rem 0;
  }}
  .timeline::before {{
    content: '';
    position: absolute;
    left: 50%;
    transform: translateX(-50%);
    width: 3px;
    height: 100%;
    background: #1a2332;
  }}
  .event {{
    position: relative;
    width: 45%;
    padding: 1rem 1.5rem;
    background: #1a2332;
    border-radius: 8px;
    margin-bottom: 1.5rem;
    border-left: 4px solid var(--accent);
  }}
  .event:nth-child(odd) {{
    margin-left: 5%;
  }}
  .event:nth-child(even) {{
    margin-left: 50%;
  }}
  .event .date {{
    font-size: 0.8rem;
    color: #4fc3f7;
    margin-bottom: 0.3rem;
  }}
  .event .title {{
    font-weight: 600;
    margin-bottom: 0.3rem;
  }}
  .event .description {{
    font-size: 0.85rem;
    color: #b0bec5;
  }}
  .event .badge {{
    display: inline-block;
    padding: 0.15rem 0.5rem;
    border-radius: 4px;
    font-size: 0.7rem;
    font-weight: 600;
    text-transform: uppercase;
    margin-top: 0.5rem;
  }}
  .type-evidence {{ --accent: #1976D2; }}
  .type-evidence .badge {{ background: #1976D2; color: #fff; }}
  .type-entity {{ --accent: #388E3C; }}
  .type-entity .badge {{ background: #388E3C; color: #fff; }}
  .type-hypothesis_update, .type-hypothesis_snapshot {{ --accent: #F57C00; }}
  .type-hypothesis_update .badge, .type-hypothesis_snapshot .badge {{ background: #F57C00; color: #fff; }}
  .type-monitoring_result {{ --accent: #7B1FA2; }}
  .type-monitoring_result .badge {{ background: #7B1FA2; color: #fff; }}
  .type-event {{ --accent: #C62828; }}
  .type-event .badge {{ background: #C62828; color: #fff; }}
  .filter-bar {{
    text-align: center;
    margin-bottom: 1.5rem;
  }}
  .filter-bar button {{
    background: #1a2332;
    border: 1px solid #2a3a4a;
    color: #e0e0e0;
    padding: 0.4rem 1rem;
    border-radius: 4px;
    cursor: pointer;
    margin: 0 0.2rem;
    font-size: 0.85rem;
  }}
  .filter-bar button:hover, .filter-bar button.active {{
    background: #4fc3f7;
    color: #0a0e17;
  }}
  .empty {{
    text-align: center;
    color: #78909c;
    padding: 3rem;
  }}
</style>
</head>
<body>
<div class="header">
  <h1>{title}</h1>
  <div class="meta">
    {case_name} {case_ref}
    <br>Genere le {generated_at} par NEXUS
  </div>
</div>
<div class="stats">
  <div class="stat-box">
    <div class="number">{event_count}</div>
    <div class="label">Evenements</div>
  </div>
</div>
<div class="filter-bar" id="filters"></div>
<div class="timeline" id="timeline"></div>

<script>
const events = {events_json};

// Collect unique types
const types = [...new Set(events.map(e => e.type || 'other'))];

// Build filter buttons
const filterBar = document.getElementById('filters');
const allBtn = document.createElement('button');
allBtn.textContent = 'Tous';
allBtn.className = 'active';
allBtn.onclick = () => render(events);
filterBar.appendChild(allBtn);

types.forEach(t => {{
  const btn = document.createElement('button');
  btn.textContent = t;
  btn.onclick = () => render(events.filter(e => e.type === t));
  filterBar.appendChild(btn);
}});

function formatDate(d) {{
  if (!d) return 'N/A';
  try {{
    const dt = new Date(d);
    return dt.toLocaleDateString('fr-FR') + ' ' + dt.toLocaleTimeString('fr-FR', {{hour:'2-digit', minute:'2-digit'}});
  }} catch(e) {{ return d; }}
}}

function render(data) {{
  const tl = document.getElementById('timeline');
  if (!data.length) {{
    tl.innerHTML = '<div class="empty">Aucun evenement a afficher</div>';
    return;
  }}
  tl.innerHTML = data.map(ev => `
    <div class="event type-${{ev.type || 'other'}}">
      <div class="date">${{formatDate(ev.date)}}</div>
      <div class="title">${{ev.title || '?'}}</div>
      <div class="description">${{ev.description || ''}}</div>
      <span class="badge">${{ev.type || 'other'}}</span>
    </div>
  `).join('');
}}

render(events);
</script>
</body>
</html>"""
