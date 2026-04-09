"""
NEXUS GOV -- Newsletter Worker (Phase 6.5).

Sends a weekly email digest with the political recap:
- Stats summary (politicians, positions, contradictions)
- Top contradictions with severity badges
- Top alerts of the week
- Key facts

Triggered by TICK_WEEKLY. Uses stdlib smtplib (no external dependency).
If SMTP is not configured (empty env vars), logs the recap and skips sending.
"""

from __future__ import annotations

import json
import os
import smtplib
from email.mime.multipart import MIMEMultipart
from email.mime.text import MIMEText
from typing import Any

from loguru import logger

from nexus.engine import _new_id, _now_iso, _row_to_dict, get_db, NexusEvent, ReactiveWorker
from nexus.gov.events import GovEventType

# SMTP configuration — all optional, graceful skip if not set
SMTP_HOST = os.environ.get("NEXUS_SMTP_HOST", "")
SMTP_PORT = int(os.environ.get("NEXUS_SMTP_PORT", "587"))
SMTP_USER = os.environ.get("NEXUS_SMTP_USER", "")
SMTP_PASS = os.environ.get("NEXUS_SMTP_PASS", "")
SMTP_FROM = os.environ.get("NEXUS_SMTP_FROM", "nexus-gov@localhost")
RECIPIENTS = os.environ.get("NEXUS_NEWSLETTER_RECIPIENTS", "")  # comma-separated


def _severity_color(severity: str) -> str:
    """Return inline CSS color for a severity level."""
    colors = {
        "critical": "#ef4444",
        "high": "#f97316",
        "medium": "#eab308",
        "low": "#22c55e",
        "info": "#64748b",
    }
    return colors.get(severity, "#94a3b8")


def _severity_badge(severity: str) -> str:
    """Return an inline-CSS HTML badge for a severity level."""
    color = _severity_color(severity)
    return (
        f'<span style="display:inline-block;padding:2px 8px;border-radius:4px;'
        f'font-size:12px;font-weight:600;color:#fff;background:{color};">'
        f"{severity.upper()}</span>"
    )


def _build_html(
    *,
    stats: dict,
    contradictions: list[dict],
    alerts: list[dict],
    recap_text: str,
    date_range: str,
) -> str:
    """Build the newsletter HTML with dark-theme inline CSS."""

    # Stats row
    pol_count = stats.get("gov_politicians", stats.get("politicians", 0))
    pos_count = stats.get("gov_positions", stats.get("positions", 0))
    contra_count = stats.get("gov_contradictions", stats.get("contradictions", 0))

    stats_html = f"""
    <table style="width:100%;border-collapse:collapse;margin:16px 0;">
      <tr>
        <td style="text-align:center;padding:12px;background:#1e293b;border-radius:8px;">
          <div style="font-size:28px;font-weight:700;color:#60a5fa;">{pol_count}</div>
          <div style="font-size:12px;color:#94a3b8;margin-top:4px;">Politiciens suivis</div>
        </td>
        <td style="width:8px;"></td>
        <td style="text-align:center;padding:12px;background:#1e293b;border-radius:8px;">
          <div style="font-size:28px;font-weight:700;color:#34d399;">{pos_count}</div>
          <div style="font-size:12px;color:#94a3b8;margin-top:4px;">Positions enregistrees</div>
        </td>
        <td style="width:8px;"></td>
        <td style="text-align:center;padding:12px;background:#1e293b;border-radius:8px;">
          <div style="font-size:28px;font-weight:700;color:#f59e0b;">{contra_count}</div>
          <div style="font-size:12px;color:#94a3b8;margin-top:4px;">Contradictions detectees</div>
        </td>
      </tr>
    </table>
    """

    # Contradictions section
    contra_rows = ""
    for c in contradictions[:5]:
        severity = c.get("severity", "info")
        subject = c.get("subject", "N/A")
        desc = c.get("description", "")[:200]
        contra_rows += f"""
        <tr>
          <td style="padding:8px 12px;border-bottom:1px solid #334155;">{_severity_badge(severity)}</td>
          <td style="padding:8px 12px;border-bottom:1px solid #334155;color:#e2e8f0;font-weight:600;">{subject}</td>
          <td style="padding:8px 12px;border-bottom:1px solid #334155;color:#94a3b8;font-size:13px;">{desc}</td>
        </tr>
        """

    contradictions_html = ""
    if contradictions:
        contradictions_html = f"""
        <h2 style="color:#f59e0b;font-size:18px;margin:24px 0 12px;">Contradictions de la semaine</h2>
        <table style="width:100%;border-collapse:collapse;background:#1e293b;border-radius:8px;overflow:hidden;">
          <thead>
            <tr style="background:#0f172a;">
              <th style="padding:10px 12px;text-align:left;color:#64748b;font-size:12px;text-transform:uppercase;">Severite</th>
              <th style="padding:10px 12px;text-align:left;color:#64748b;font-size:12px;text-transform:uppercase;">Sujet</th>
              <th style="padding:10px 12px;text-align:left;color:#64748b;font-size:12px;text-transform:uppercase;">Description</th>
            </tr>
          </thead>
          <tbody>
            {contra_rows}
          </tbody>
        </table>
        """

    # Alerts section
    alert_rows = ""
    for a in alerts[:5]:
        severity = a.get("severity", "info")
        title = a.get("title", "N/A")
        desc = a.get("description", "")[:150]
        alert_rows += f"""
        <tr>
          <td style="padding:8px 12px;border-bottom:1px solid #334155;">{_severity_badge(severity)}</td>
          <td style="padding:8px 12px;border-bottom:1px solid #334155;color:#e2e8f0;">{title}</td>
          <td style="padding:8px 12px;border-bottom:1px solid #334155;color:#94a3b8;font-size:13px;">{desc}</td>
        </tr>
        """

    alerts_html = ""
    if alerts:
        alerts_html = f"""
        <h2 style="color:#60a5fa;font-size:18px;margin:24px 0 12px;">Alertes de la semaine</h2>
        <table style="width:100%;border-collapse:collapse;background:#1e293b;border-radius:8px;overflow:hidden;">
          <thead>
            <tr style="background:#0f172a;">
              <th style="padding:10px 12px;text-align:left;color:#64748b;font-size:12px;text-transform:uppercase;">Niveau</th>
              <th style="padding:10px 12px;text-align:left;color:#64748b;font-size:12px;text-transform:uppercase;">Titre</th>
              <th style="padding:10px 12px;text-align:left;color:#64748b;font-size:12px;text-transform:uppercase;">Description</th>
            </tr>
          </thead>
          <tbody>
            {alert_rows}
          </tbody>
        </table>
        """

    # Full HTML
    html = f"""<!DOCTYPE html>
<html>
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
</head>
<body style="margin:0;padding:0;background:#0f172a;font-family:Arial,Helvetica,sans-serif;">
  <table style="width:100%;max-width:640px;margin:0 auto;background:#0f172a;">
    <tr>
      <td style="padding:32px 24px;">
        <!-- Header -->
        <table style="width:100%;margin-bottom:24px;">
          <tr>
            <td>
              <h1 style="margin:0;font-size:24px;color:#f8fafc;">NEXUS GOV</h1>
              <p style="margin:4px 0 0;font-size:14px;color:#64748b;">Alerte Politique Hebdomadaire</p>
            </td>
            <td style="text-align:right;color:#64748b;font-size:13px;">{date_range}</td>
          </tr>
        </table>

        <hr style="border:none;border-top:1px solid #334155;margin:0 0 24px;">

        {stats_html}
        {contradictions_html}
        {alerts_html}

        <!-- Footer -->
        <hr style="border:none;border-top:1px solid #334155;margin:32px 0 16px;">
        <p style="color:#475569;font-size:12px;text-align:center;margin:0;">
          Genere automatiquement par NEXUS GOV — Systeme de veille politique autonome
        </p>
      </td>
    </tr>
  </table>
</body>
</html>"""
    return html


class GovNewsletterWorker(ReactiveWorker):
    """Sends weekly email digest with the political recap."""

    name = "gov_newsletter"
    subscriptions = [GovEventType.TICK_WEEKLY]

    def __init__(self, bus: Any, db: Any) -> None:
        super().__init__(bus)
        self._db = db

    async def handle(self, event: NexusEvent) -> list[NexusEvent]:
        output: list[NexusEvent] = []

        try:
            # Fetch stats
            stats = await self._db.get_stats()

            # Fetch latest recap alert
            async with get_db() as conn:
                cursor = await conn.execute(
                    "SELECT * FROM gov_alerts WHERE alert_type = 'recap' "
                    "ORDER BY created_at DESC LIMIT 1"
                )
                recap_row = await cursor.fetchone()
                recap_text = ""
                if recap_row:
                    recap_dict = _row_to_dict(recap_row)
                    recap_text = recap_dict.get("description", "")

                # Top 5 contradictions of the week (last 7 days)
                cursor = await conn.execute(
                    "SELECT * FROM gov_contradictions "
                    "WHERE detected_at >= datetime('now', '-7 days') "
                    "ORDER BY CASE severity "
                    "  WHEN 'critical' THEN 1 "
                    "  WHEN 'high' THEN 2 "
                    "  WHEN 'medium' THEN 3 "
                    "  WHEN 'low' THEN 4 "
                    "  ELSE 5 END, detected_at DESC "
                    "LIMIT 5"
                )
                contradictions = [_row_to_dict(r) for r in await cursor.fetchall()]

                # Top 5 alerts of the week (excluding recaps)
                cursor = await conn.execute(
                    "SELECT * FROM gov_alerts "
                    "WHERE alert_type != 'recap' "
                    "AND created_at >= datetime('now', '-7 days') "
                    "ORDER BY CASE severity "
                    "  WHEN 'critical' THEN 1 "
                    "  WHEN 'high' THEN 2 "
                    "  WHEN 'medium' THEN 3 "
                    "  WHEN 'low' THEN 4 "
                    "  ELSE 5 END, created_at DESC "
                    "LIMIT 5"
                )
                alerts = [_row_to_dict(r) for r in await cursor.fetchall()]

            # Build date range string
            from datetime import datetime, timedelta, timezone

            now = datetime.now(timezone.utc)
            week_ago = now - timedelta(days=7)
            date_range = f"{week_ago.strftime('%d/%m/%Y')} — {now.strftime('%d/%m/%Y')}"

            # Build HTML email body
            html_body = _build_html(
                stats=stats,
                contradictions=contradictions,
                alerts=alerts,
                recap_text=recap_text,
                date_range=date_range,
            )

            # Attempt SMTP send
            send_status = "skipped"
            send_error = ""

            if SMTP_HOST and RECIPIENTS:
                try:
                    send_status = await self._send_email(html_body, date_range)
                except Exception as exc:
                    send_status = "failed"
                    send_error = str(exc)
                    logger.error("Newsletter SMTP send failed: {}", exc)
            else:
                logger.info(
                    "Newsletter: SMTP not configured (NEXUS_SMTP_HOST='{}', "
                    "NEXUS_NEWSLETTER_RECIPIENTS='{}'), skipping send",
                    SMTP_HOST,
                    RECIPIENTS,
                )

            # Store send status as an alert
            async with get_db() as conn:
                alert_id = _new_id()
                now_iso = _now_iso()
                metadata = json.dumps({
                    "send_status": send_status,
                    "send_error": send_error,
                    "recipients": RECIPIENTS,
                    "contradictions_count": len(contradictions),
                    "alerts_count": len(alerts),
                }, ensure_ascii=False)

                await conn.execute(
                    """INSERT INTO gov_alerts
                       (id, alert_type, title, description, severity, created_at)
                       VALUES (?, ?, ?, ?, ?, ?)""",
                    (
                        alert_id,
                        "newsletter",
                        f"Newsletter hebdomadaire ({send_status})",
                        f"Newsletter envoyee: {len(contradictions)} contradictions, "
                        f"{len(alerts)} alertes. Statut: {send_status}",
                        "info",
                        now_iso,
                    ),
                )
                await conn.commit()

            output.append(
                NexusEvent(
                    event_type=GovEventType.GOV_ALERT_CREATED,
                    case_id="gov",
                    payload={
                        "alert_id": alert_id,
                        "alert_type": "newsletter",
                        "title": f"Newsletter hebdomadaire ({send_status})",
                        "send_status": send_status,
                    },
                    source_worker=self.name,
                    parent_event_id=event.event_id,
                )
            )

            logger.info(
                "Newsletter generated: {} contradictions, {} alerts, send={}",
                len(contradictions),
                len(alerts),
                send_status,
            )

        except Exception as exc:
            logger.error("Newsletter worker failed: {}", exc)

        return output

    async def _send_email(self, html_body: str, date_range: str) -> str:
        """Send the newsletter via SMTP. Returns 'sent' on success."""
        import asyncio

        # Run blocking SMTP in executor to avoid blocking the event loop
        loop = asyncio.get_running_loop()
        await loop.run_in_executor(
            None, self._smtp_send_sync, html_body, date_range
        )
        return "sent"

    @staticmethod
    def _smtp_send_sync(html_body: str, date_range: str) -> None:
        """Synchronous SMTP send (runs in thread executor)."""
        recipient_list = [r.strip() for r in RECIPIENTS.split(",") if r.strip()]
        if not recipient_list:
            return

        msg = MIMEMultipart("alternative")
        msg["Subject"] = f"NEXUS GOV — Alerte Politique Hebdomadaire ({date_range})"
        msg["From"] = SMTP_FROM
        msg["To"] = ", ".join(recipient_list)

        # Plain text fallback
        plain_text = (
            f"NEXUS GOV — Alerte Politique Hebdomadaire\n"
            f"Periode: {date_range}\n\n"
            "Consultez la version HTML de cet email pour le rapport complet.\n\n"
            "Genere automatiquement par NEXUS GOV"
        )
        msg.attach(MIMEText(plain_text, "plain", "utf-8"))
        msg.attach(MIMEText(html_body, "html", "utf-8"))

        with smtplib.SMTP(SMTP_HOST, SMTP_PORT, timeout=30) as server:
            server.ehlo()
            if SMTP_PORT != 25:
                server.starttls()
                server.ehlo()
            if SMTP_USER and SMTP_PASS:
                server.login(SMTP_USER, SMTP_PASS)
            server.sendmail(SMTP_FROM, recipient_list, msg.as_string())

        logger.info(
            "Newsletter sent to {} recipients via {}:{}",
            len(recipient_list),
            SMTP_HOST,
            SMTP_PORT,
        )
