#!/usr/bin/env python3
"""Prepend a dated workflow note to the retained SharePoint example report."""

from io import BytesIO
from pathlib import Path

from pypdf import PdfReader, PdfWriter
from reportlab.lib.colors import HexColor
from reportlab.lib.styles import ParagraphStyle
from reportlab.pdfgen.canvas import Canvas
from reportlab.platypus import Paragraph


ASSETS = Path(__file__).resolve().parent
REPORT = ASSETS / "sharepoint/demo-20261026/sharepoint-demo-report.pdf"
OUTPUT = ASSETS.parents[3] / "output/pdf/example_report.pdf"
INK = HexColor("#163C47")
GREEN = HexColor("#00785B")


def note_page() -> BytesIO:
    buffer = BytesIO()
    canvas = Canvas(buffer, pagesize=(612, 792))
    canvas.setTitle("Sponsor Highlights - Example report and workflow note")
    canvas.setFillColor(GREEN)
    canvas.rect(0, 696, 612, 96, fill=1, stroke=0)
    canvas.setFillColor(HexColor("#FFFFFF"))
    canvas.setFont("Helvetica-Bold", 24)
    canvas.drawString(48, 744, "Example report")
    canvas.setFont("Helvetica", 12)
    canvas.drawString(48, 718, "Workflow update  |  October 29, 2026")
    style = ParagraphStyle("copy", fontName="Helvetica", fontSize=12, leading=18, textColor=INK)
    paragraphs = (
        "The next three pages are the retained October 26, 2026, TEST ONLY SharePoint report. "
        "They remain a dated example of reviewed local reporting, not evidence of a calendar invitation.",
        "A separate automated flow is now On. The first original project highlight saved "
        "before its month's second Friday in Eastern time can invite the verified submitter "
        "to a free, all-day Outlook reminder. A pre-deadline correction of that first highlight "
        "can also qualify if no matching reminder exists. Highlights and reviews are due beforehand.",
        "Later highlights and their corrections, general updates, and receipts processed on or "
        "after that Friday skip the invite. The actual submission time sets the month; editing "
        "the highlight date does not. The calendar duplicate check is not an atomic identity ledger.",
        "The flow passed its checker with zero errors and warnings on October 29. No qualifying "
        "run or attendee delivery had been observed when this note was issued.",
    )
    top = 646
    for content in paragraphs:
        paragraph = Paragraph(content, style)
        _, height = paragraph.wrap(516, 700)
        paragraph.drawOn(canvas, 48, top - height)
        top -= height + 22
    canvas.setStrokeColor(HexColor("#C4D9DB"))
    canvas.line(48, 88, 564, 88)
    canvas.setFillColor(INK)
    canvas.setFont("Helvetica", 9)
    canvas.drawString(48, 70, "ACORN / ORNL  |  TEST ONLY  |  WORKFLOW NOTE")
    canvas.save()
    buffer.seek(0)
    return buffer


def main() -> None:
    writer = PdfWriter()
    writer.append(PdfReader(note_page()))
    writer.append(PdfReader(REPORT))
    writer.add_metadata({"/Title": "Sponsor Highlights - Example report and workflow note", "/Author": "ACORN / ORNL"})
    OUTPUT.parent.mkdir(parents=True, exist_ok=True)
    with OUTPUT.open("wb") as target:
        writer.write(target)


if __name__ == "__main__":
    main()
