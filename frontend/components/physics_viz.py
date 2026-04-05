"""
NEXUS -- Physics simulation visualisation components.

Plotly-based renderers for forensic physics simulations:
- Blood drop trajectory (3D)
- Cast-off pattern (2D top-down view)
- Sound propagation (2D map with wavefronts)
- Impact angle diagram
- Origin-of-impact convergence
"""

from __future__ import annotations

import math
from typing import Any

import numpy as np
import plotly.graph_objects as go
from plotly.subplots import make_subplots


# ===================================================================
# Blood drop trajectory — 3D
# ===================================================================

def render_blood_trajectory(sim_result: dict[str, Any]) -> go.Figure:
    """Render a 3D trajectory of a single blood drop.

    Parameters
    ----------
    sim_result : dict
        Output from ForensicPhysicsSim.simulate_blood_drop().

    Returns
    -------
    plotly.graph_objects.Figure
    """
    traj = sim_result["trajectory"]
    xs = [p[0] for p in traj]
    ys = [p[2] for p in traj]  # z -> lateral (unused, 0)
    zs = [p[1] for p in traj]  # y -> height

    fig = go.Figure()

    # Trajectory line
    fig.add_trace(go.Scatter3d(
        x=xs, y=ys, z=zs,
        mode="lines",
        line=dict(color="crimson", width=4),
        name="Trajectoire",
    ))

    # Release point
    fig.add_trace(go.Scatter3d(
        x=[xs[0]], y=[ys[0]], z=[zs[0]],
        mode="markers",
        marker=dict(size=8, color="blue", symbol="diamond"),
        name="Point de depart",
    ))

    # Impact point
    impact = sim_result["impact_point"]
    fig.add_trace(go.Scatter3d(
        x=[impact[0]], y=[0], z=[0],
        mode="markers",
        marker=dict(size=10, color="red", symbol="x"),
        name="Point d'impact",
    ))

    # Stain ellipse on the ground plane
    stain = sim_result["stain_shape"]
    w_m = stain["width_mm"] / 1000.0
    l_m = stain["length_mm"] / 1000.0
    theta = np.linspace(0, 2 * np.pi, 40)
    stain_x = impact[0] + (l_m / 2) * np.cos(theta)
    stain_y = (w_m / 2) * np.sin(theta)
    stain_z = np.zeros_like(theta)

    fig.add_trace(go.Scatter3d(
        x=stain_x.tolist(), y=stain_y.tolist(), z=stain_z.tolist(),
        mode="lines",
        line=dict(color="darkred", width=3),
        name=f"Tache ({stain['width_mm']:.1f} x {stain['length_mm']:.1f} mm)",
    ))

    # Annotations
    impact_angle = sim_result["impact_angle"]
    travel_time = sim_result["travel_time"]
    impact_vel = sim_result["impact_velocity"]

    fig.update_layout(
        title=dict(
            text=(
                f"Trajectoire de goutte de sang | "
                f"Angle d'impact: {impact_angle:.1f} deg | "
                f"Vitesse d'impact: {impact_vel:.2f} m/s | "
                f"Temps: {travel_time*1000:.1f} ms"
            ),
            font=dict(size=13),
        ),
        scene=dict(
            xaxis_title="Distance horizontale (m)",
            yaxis_title="Lateral (m)",
            zaxis_title="Hauteur (m)",
            aspectmode="data",
        ),
        showlegend=True,
        legend=dict(x=0, y=1),
        margin=dict(l=0, r=0, t=60, b=0),
    )

    return fig


# ===================================================================
# Cast-off pattern — 2D top-down
# ===================================================================

def render_cast_off_pattern(drops: list[dict[str, Any]]) -> go.Figure:
    """Render a 2D top-down view of a cast-off blood pattern.

    Parameters
    ----------
    drops : list of dict
        Output from ForensicPhysicsSim.simulate_cast_off().

    Returns
    -------
    plotly.graph_objects.Figure
    """
    fig = make_subplots(
        rows=1, cols=2,
        subplot_titles=("Vue du dessus (impact)", "Vue laterale (trajectoires)"),
        column_widths=[0.5, 0.5],
    )

    # Colour scale by release angle
    if drops:
        angles = [d["release_angle_deg"] for d in drops]
        min_a, max_a = min(angles), max(angles)
        range_a = max_a - min_a if max_a > min_a else 1.0
    else:
        min_a, range_a = 0, 1

    # -- Left panel: impact points (top-down) --
    impact_xs = []
    impact_ys = []
    colors = []
    hover_texts = []

    for d in drops:
        imp = d["drop_sim"]["impact_point"]
        impact_xs.append(imp[0])
        impact_ys.append(0)  # all on ground (y=0 for top view, x=horizontal)
        norm = (d["release_angle_deg"] - min_a) / range_a
        colors.append(norm)
        stain = d["drop_sim"]["stain_shape"]
        hover_texts.append(
            f"Drop {d['drop_index']}<br>"
            f"Release: {d['release_angle_deg']:.0f} deg<br>"
            f"v_tang: {d['tangential_velocity']:.1f} m/s<br>"
            f"Stain: {stain['width_mm']:.1f} x {stain['length_mm']:.1f} mm<br>"
            f"Impact angle: {d['drop_sim']['impact_angle']:.1f} deg"
        )

    fig.add_trace(
        go.Scatter(
            x=impact_xs, y=impact_ys,
            mode="markers",
            marker=dict(
                size=10,
                color=colors,
                colorscale="Reds",
                showscale=True,
                colorbar=dict(title="Angle", x=0.45),
            ),
            text=hover_texts,
            hoverinfo="text",
            name="Impacts",
        ),
        row=1, col=1,
    )

    # Draw stain ellipses (top-down: x vs lateral)
    for d in drops:
        imp = d["drop_sim"]["impact_point"]
        stain = d["drop_sim"]["stain_shape"]
        w = stain["width_mm"] / 1000.0
        l_val = stain["length_mm"] / 1000.0
        theta = np.linspace(0, 2 * np.pi, 20)
        ex = imp[0] + (l_val / 2) * np.cos(theta)
        ey = (w / 2) * np.sin(theta)
        fig.add_trace(
            go.Scatter(
                x=ex.tolist(), y=ey.tolist(),
                mode="lines",
                line=dict(color="darkred", width=1),
                showlegend=False,
                hoverinfo="skip",
            ),
            row=1, col=1,
        )

    # -- Right panel: side view (trajectories) --
    for i, d in enumerate(drops):
        traj = d["drop_sim"]["trajectory"]
        txs = [p[0] for p in traj]
        tzs = [p[1] for p in traj]
        show = i == 0
        fig.add_trace(
            go.Scatter(
                x=txs, y=tzs,
                mode="lines",
                line=dict(width=1.5, color=f"rgba(180, 30, 30, {0.3 + 0.7 * (i / max(len(drops)-1, 1))})"),
                showlegend=show,
                name="Trajectoires" if show else None,
                hoverinfo="skip",
            ),
            row=1, col=2,
        )

    # Release points
    rel_xs = [d["release_position"][0] for d in drops]
    rel_ys = [d["release_position"][1] for d in drops]
    fig.add_trace(
        go.Scatter(
            x=rel_xs, y=rel_ys,
            mode="markers",
            marker=dict(size=5, color="blue"),
            name="Points de largage",
        ),
        row=1, col=2,
    )

    fig.update_xaxes(title_text="X (m)", row=1, col=1)
    fig.update_yaxes(title_text="Lateral (m)", row=1, col=1, scaleanchor="x")
    fig.update_xaxes(title_text="X (m)", row=1, col=2)
    fig.update_yaxes(title_text="Hauteur (m)", row=1, col=2)

    fig.update_layout(
        title=f"Pattern cast-off | {len(drops)} gouttes detachees",
        height=500,
        showlegend=True,
    )

    return fig


# ===================================================================
# Sound propagation — 2D map
# ===================================================================

def render_sound_propagation(sim_result: dict[str, Any]) -> go.Figure:
    """Render a 2D sound propagation map with wavefronts and listener markers.

    Parameters
    ----------
    sim_result : dict
        Output from ForensicPhysicsSim.simulate_sound_propagation().

    Returns
    -------
    plotly.graph_objects.Figure
    """
    arrivals = sim_result["arrivals"]
    c = sim_result["speed_of_sound"]

    fig = go.Figure()

    # Source marker (placed at 0, 0 by convention if not given directly)
    # We infer source position as the origin; listeners have absolute coords.
    # For plotting, use listener positions relative to what the API returned.
    fig.add_trace(go.Scatter(
        x=[0], y=[0],
        mode="markers+text",
        marker=dict(size=14, color="red", symbol="star"),
        text=["Source"],
        textposition="top center",
        name=f"Source ({sim_result['source_level_db']} dB)",
    ))

    # Wavefront circles: show every 50 m propagation distance
    if arrivals:
        max_dist = max(a["distance_m"] for a in arrivals)
    else:
        max_dist = 100.0
    wavefront_step = max(10, round(max_dist / 6, -1))  # round to 10s

    theta = np.linspace(0, 2 * np.pi, 100)
    r = wavefront_step
    while r <= max_dist * 1.2:
        wx = (r * np.cos(theta)).tolist()
        wy = (r * np.sin(theta)).tolist()
        delay_ms = (r / c) * 1000
        fig.add_trace(go.Scatter(
            x=wx, y=wy,
            mode="lines",
            line=dict(color="rgba(100, 100, 255, 0.3)", width=1, dash="dot"),
            showlegend=False,
            hoverinfo="text",
            text=f"{r:.0f} m | {delay_ms:.0f} ms",
        ))
        r += wavefront_step

    # Listener markers, sized/colored by loudness
    lx = [a["position"][0] for a in arrivals]
    ly = [a["position"][1] for a in arrivals]
    loudness = [a["estimated_loudness_db"] for a in arrivals]
    delays = [a["delay_sec"] * 1000 for a in arrivals]

    hover = [
        (
            f"Listener {a['listener_id']}<br>"
            f"Distance: {a['distance_m']:.1f} m<br>"
            f"Delai: {a['delay_sec']*1000:.1f} ms<br>"
            f"Volume: {a['estimated_loudness_db']:.1f} dB<br>"
            f"Attenuation: {a['attenuation_db']:.1f} dB<br>"
            f"{'AUDIBLE' if a['above_hearing_threshold'] else 'INAUDIBLE'}"
            f"{' | DOULOUREUX' if a.get('above_pain_threshold') else ''}"
        )
        for a in arrivals
    ]

    fig.add_trace(go.Scatter(
        x=lx, y=ly,
        mode="markers+text",
        marker=dict(
            size=12,
            color=loudness,
            colorscale="YlOrRd",
            reversescale=False,
            showscale=True,
            colorbar=dict(title="dB SPL"),
            line=dict(width=1, color="black"),
        ),
        text=[f"L{a['listener_id']}" for a in arrivals],
        textposition="top center",
        hovertext=hover,
        hoverinfo="text",
        name="Auditeurs",
    ))

    # Lines from source to each listener
    for a in arrivals:
        opacity = 0.6 if a["above_hearing_threshold"] else 0.15
        fig.add_trace(go.Scatter(
            x=[0, a["position"][0]],
            y=[0, a["position"][1]],
            mode="lines",
            line=dict(
                color=f"rgba(200, 50, 50, {opacity})",
                width=1,
                dash="dash" if not a["above_hearing_threshold"] else "solid",
            ),
            showlegend=False,
            hoverinfo="skip",
        ))

    fig.update_layout(
        title=(
            f"Propagation sonore | c = {c:.1f} m/s | "
            f"f = {sim_result['frequency_hz']:.0f} Hz | "
            f"Terrain: {sim_result['terrain']}"
        ),
        xaxis_title="X (m)",
        yaxis_title="Y (m)",
        xaxis=dict(scaleanchor="y", scaleratio=1),
        showlegend=True,
        height=600,
    )

    return fig


# ===================================================================
# Impact angle diagram
# ===================================================================

def render_impact_angle_diagram(
    width_mm: float,
    length_mm: float,
    impact_angle: float,
) -> go.Figure:
    """Render a diagram showing the relationship between stain ellipse
    and impact angle.

    Parameters
    ----------
    width_mm, length_mm : float
        Dimensions of the elliptical bloodstain in mm.
    impact_angle : float
        Impact angle in degrees from the surface.

    Returns
    -------
    plotly.graph_objects.Figure
    """
    fig = make_subplots(
        rows=1, cols=2,
        subplot_titles=("Tache elliptique (vue du dessus)", "Angle d'impact (coupe)"),
        column_widths=[0.5, 0.5],
    )

    # -- Left: elliptical stain --
    theta = np.linspace(0, 2 * np.pi, 100)
    ex = (length_mm / 2) * np.cos(theta)
    ey = (width_mm / 2) * np.sin(theta)

    fig.add_trace(
        go.Scatter(
            x=ex.tolist(), y=ey.tolist(),
            mode="lines",
            fill="toself",
            fillcolor="rgba(180, 20, 20, 0.6)",
            line=dict(color="darkred", width=2),
            name="Tache",
        ),
        row=1, col=1,
    )

    # Width and length annotations
    fig.add_trace(
        go.Scatter(
            x=[-length_mm / 2, length_mm / 2], y=[0, 0],
            mode="lines+text",
            line=dict(color="blue", width=1, dash="dash"),
            text=[None, f"L = {length_mm:.1f} mm"],
            textposition="top right",
            showlegend=False,
        ),
        row=1, col=1,
    )
    fig.add_trace(
        go.Scatter(
            x=[0, 0], y=[-width_mm / 2, width_mm / 2],
            mode="lines+text",
            line=dict(color="green", width=1, dash="dash"),
            text=[None, f"W = {width_mm:.1f} mm"],
            textposition="top right",
            showlegend=False,
        ),
        row=1, col=1,
    )

    # -- Right: side view showing impact angle --
    # Draw the surface line
    fig.add_trace(
        go.Scatter(
            x=[-2, 2], y=[0, 0],
            mode="lines",
            line=dict(color="gray", width=3),
            name="Surface",
        ),
        row=1, col=2,
    )

    # Incoming trajectory line
    angle_rad = math.radians(impact_angle)
    arrow_len = 1.5
    ax = -arrow_len * math.cos(angle_rad)
    ay = arrow_len * math.sin(angle_rad)
    fig.add_trace(
        go.Scatter(
            x=[ax, 0], y=[ay, 0],
            mode="lines+markers",
            line=dict(color="crimson", width=3),
            marker=dict(size=[0, 8], color="crimson", symbol=["circle", "arrow-down"]),
            name="Trajectoire",
        ),
        row=1, col=2,
    )

    # Angle arc
    arc_angles = np.linspace(0, angle_rad, 20)
    arc_r = 0.5
    arc_x = arc_r * np.cos(np.pi - arc_angles)
    arc_y = arc_r * np.sin(np.pi - arc_angles)
    fig.add_trace(
        go.Scatter(
            x=arc_x.tolist(), y=arc_y.tolist(),
            mode="lines",
            line=dict(color="orange", width=2),
            showlegend=False,
        ),
        row=1, col=2,
    )

    # Angle label
    label_angle = angle_rad / 2
    fig.add_annotation(
        x=-arc_r * 1.4 * math.cos(label_angle),
        y=arc_r * 1.4 * math.sin(label_angle),
        text=f"<b>{impact_angle:.1f} deg</b>",
        showarrow=False,
        font=dict(size=14, color="orange"),
        xref="x2", yref="y2",
    )

    # Formula annotation
    sin_val = width_mm / length_mm if length_mm > 0 else 0
    fig.add_annotation(
        x=0, y=-0.5,
        text=f"sin(alpha) = W/L = {width_mm:.1f}/{length_mm:.1f} = {sin_val:.3f}",
        showarrow=False,
        font=dict(size=11),
        xref="x2", yref="y2",
    )

    fig.update_xaxes(scaleanchor="y", row=1, col=1)
    fig.update_yaxes(scaleanchor="x", row=1, col=2)

    fig.update_layout(
        title=f"Angle d'impact: {impact_angle:.1f} deg | sin(alpha) = W/L = {sin_val:.3f}",
        height=400,
        showlegend=True,
    )

    return fig


# ===================================================================
# Origin of impact — convergence diagram
# ===================================================================

def render_origin_convergence(
    origin_result: dict[str, Any],
    stains: list[dict[str, Any]],
) -> go.Figure:
    """Render the convergence lines and estimated origin of impact.

    Parameters
    ----------
    origin_result : dict
        Output from ForensicPhysicsSim.estimate_origin_of_impact().
    stains : list of dict
        The original stain measurements.

    Returns
    -------
    plotly.graph_objects.Figure
    """
    fig = go.Figure()

    # Stain positions
    sx = [s["x"] for s in stains]
    sy = [s["y"] for s in stains]
    fig.add_trace(go.Scatter(
        x=sx, y=sy,
        mode="markers",
        marker=dict(size=10, color="darkred", symbol="circle"),
        name="Taches",
        text=[
            f"Stain {i}<br>W={s['width_mm']:.1f} mm<br>L={s['length_mm']:.1f} mm"
            for i, s in enumerate(stains)
        ],
        hoverinfo="text",
    ))

    # Convergence lines
    origin_x = origin_result["origin_x"]
    origin_y = origin_result["origin_y"]
    lines = origin_result.get("convergence_lines", [])

    for i, ln in enumerate(lines):
        # Draw line from stain toward (and past) the origin
        ext = 1.5  # extend past origin for visibility
        end_x = ln["x"] + ln["direction_dx"] * 20
        end_y = ln["y"] + ln["direction_dy"] * 20
        fig.add_trace(go.Scatter(
            x=[ln["x"], end_x],
            y=[ln["y"], end_y],
            mode="lines",
            line=dict(color="rgba(50, 50, 200, 0.4)", width=1, dash="dash"),
            showlegend=i == 0,
            name="Lignes de convergence" if i == 0 else None,
            hoverinfo="skip",
        ))

    # Estimated origin
    fig.add_trace(go.Scatter(
        x=[origin_x], y=[origin_y],
        mode="markers+text",
        marker=dict(size=14, color="gold", symbol="star", line=dict(width=2, color="black")),
        text=[f"Origine ({origin_x:.2f}, {origin_y:.2f})"],
        textposition="top center",
        name=f"Origine estimee (z={origin_result['origin_z']:.2f} m)",
    ))

    fig.update_layout(
        title=(
            f"Point de convergence | "
            f"Origine: ({origin_x:.2f}, {origin_y:.2f}) | "
            f"Hauteur: {origin_result['origin_z']:.2f} m | "
            f"Residuel: {origin_result['residual_m']:.3f} m"
        ),
        xaxis_title="X (m)",
        yaxis_title="Y (m)",
        xaxis=dict(scaleanchor="y", scaleratio=1),
        height=500,
        showlegend=True,
    )

    return fig
