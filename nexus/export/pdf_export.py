"""
NEXUS -- PDF export via WeasyPrint + Jinja2.

Renders investigation reports to HTML from Jinja2 templates,
then converts to PDF using WeasyPrint.
"""

from __future__ import annotations

from pathlib import Path
from typing import Any, Dict, Optional

from jinja2 import Environment, FileSystemLoader, select_autoescape
from loguru import logger


class PDFExporter:
    """Render report data to HTML and export as PDF.

    Usage::

        exporter = PDFExporter(templates_dir=Path("nexus/export/templates"))
        html = exporter.render_to_html("full_report.html", report_data)
        exporter.export_to_pdf(html, Path("output/report.pdf"))
    """

    def __init__(self, templates_dir: Optional[Path] = None) -> None:
        if templates_dir is None:
            templates_dir = Path(__file__).parent / "templates"

        self._templates_dir = templates_dir
        self._env = Environment(
            loader=FileSystemLoader(str(templates_dir)),
            autoescape=select_autoescape(["html"]),
        )
        # Register custom filters
        self._env.filters["format_score"] = self._format_score
        self._env.filters["format_date"] = self._format_date
        self._env.filters["severity_color"] = self._severity_color

        logger.debug("PDFExporter initialised (templates={})", templates_dir)

    # ------------------------------------------------------------------
    # Public API
    # ------------------------------------------------------------------

    def render_to_html(
        self,
        template_name: str,
        data: Dict[str, Any],
    ) -> str:
        """Render a Jinja2 template to an HTML string.

        Parameters
        ----------
        template_name : str
            Name of the template file (e.g. "full_report.html").
        data : dict
            Template context data (the report dict).

        Returns
        -------
        str
            Rendered HTML string.
        """
        template = self._env.get_template(template_name)
        html = template.render(**data)
        logger.debug(
            "Rendered template '{}' ({} chars)",
            template_name,
            len(html),
        )
        return html

    def export_to_pdf(self, html: str, output_path: Path) -> Path:
        """Convert an HTML string to a PDF file using WeasyPrint.

        Parameters
        ----------
        html : str
            Rendered HTML content.
        output_path : Path
            Destination file path for the PDF.

        Returns
        -------
        Path
            The output path (same as input, for convenience).
        """
        try:
            from weasyprint import HTML as WeasyHTML
        except ImportError:
            logger.error(
                "WeasyPrint is not installed. "
                "Install with: pip install weasyprint"
            )
            raise ImportError(
                "WeasyPrint is required for PDF export. "
                "Install with: pip install weasyprint"
            )

        output_path.parent.mkdir(parents=True, exist_ok=True)

        logger.info("Exporting PDF to {}", output_path)
        doc = WeasyHTML(string=html)
        doc.write_pdf(str(output_path))

        file_size = output_path.stat().st_size
        logger.info(
            "PDF exported: {} ({} bytes)",
            output_path,
            file_size,
        )
        return output_path

    def export_report(
        self,
        report_data: Dict[str, Any],
        output_path: Path,
        *,
        template_name: str = "full_report.html",
    ) -> Path:
        """Convenience method: render template + export to PDF in one call.

        Parameters
        ----------
        report_data : dict
            The report data dict (from ReportGenerator).
        output_path : Path
            Destination file path for the PDF.
        template_name : str
            Template to use (defaults to "full_report.html").

        Returns
        -------
        Path
            The output path.
        """
        html = self.render_to_html(template_name, report_data)
        return self.export_to_pdf(html, output_path)

    # ------------------------------------------------------------------
    # Jinja2 custom filters
    # ------------------------------------------------------------------

    @staticmethod
    def _format_score(value: Any) -> str:
        """Format a numeric score as 'XX.X/100'."""
        try:
            return f"{float(value):.1f}/100"
        except (TypeError, ValueError):
            return str(value)

    @staticmethod
    def _format_date(value: Any) -> str:
        """Format an ISO date string as 'DD/MM/YYYY HH:MM'."""
        if not value:
            return "N/A"
        s = str(value)
        # Try to parse ISO format
        try:
            from datetime import datetime
            if "T" in s:
                dt = datetime.fromisoformat(s.replace("Z", "+00:00"))
            else:
                dt = datetime.fromisoformat(s)
            return dt.strftime("%d/%m/%Y %H:%M")
        except (ValueError, TypeError):
            return s

    @staticmethod
    def _severity_color(severity: str) -> str:
        """Map alert severity to a CSS color."""
        colors = {
            "info": "#2196F3",
            "warning": "#FF9800",
            "critical": "#F44336",
        }
        return colors.get(severity, "#757575")
