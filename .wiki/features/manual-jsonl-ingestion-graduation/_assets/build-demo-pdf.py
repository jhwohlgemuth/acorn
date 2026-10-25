#!/usr/bin/env python3
"""Render the current demo walkthrough and retained screenshots as a branded PDF."""
import html
from io import BytesIO
from pathlib import Path
import re
import textwrap

from reportlab.lib import colors
from pypdf import PdfReader, PdfWriter
from reportlab.lib.styles import ParagraphStyle, getSampleStyleSheet
from reportlab.platypus import Image, KeepTogether, PageBreak, Paragraph, SimpleDocTemplate, Spacer, Table, TableStyle
from reportlab.pdfgen.canvas import Canvas

ASSETS = Path(__file__).resolve().parent
SOURCE = ASSETS.parent / "walkthrough.md"
OUTPUT = ASSETS / "demo-walkthrough.pdf"
GREEN = colors.HexColor("#00785B")
INK = colors.HexColor("#163643")
PALE = colors.HexColor("#EAF3F1")
WIDTH = 504


def inline(value):
    escaped = html.escape(value)
    def link(match):
        label, target = match.groups()
        if "://" not in target:
            path, _, anchor = html.unescape(target).partition("#")
            target = (SOURCE.parent / path).resolve().as_uri() + ("#" + anchor if anchor else "")
        return f'<a href="{html.escape(target, quote=True)}" color="#00785B">{label}</a>'
    escaped = re.sub(r"\[([^\]]+)\]\(([^)]+)\)", link, escaped)
    escaped = re.sub(r"`([^`]+)`", r'<font name="Courier" size="8.5">\1</font>', escaped)
    return re.sub(r"\*\*([^*]+)\*\*", r"<b>\1</b>", escaped)


def main():
    styles = getSampleStyleSheet()
    styles.add(ParagraphStyle("Body", fontName="Helvetica", fontSize=10, leading=14.5, textColor=INK, spaceAfter=8))
    styles.add(ParagraphStyle("Section", parent=styles["Body"], fontName="Helvetica-Bold", fontSize=16, leading=20,
                              textColor=GREEN, spaceBefore=14, spaceAfter=9, keepWithNext=True))
    styles.add(ParagraphStyle("Cell", parent=styles["Body"], fontSize=8.5, leading=11.5, spaceAfter=0))
    styles.add(ParagraphStyle("DemoCode", parent=styles["Body"], fontName="Courier", fontSize=7.6, leading=10.5,
                              backColor=PALE, borderPadding=9, spaceBefore=7, spaceAfter=12))
    styles.add(ParagraphStyle("Caption", parent=styles["Body"], fontSize=8, leading=11, textColor=colors.HexColor("#526A70")))
    styles.add(ParagraphStyle("Hero", parent=styles["Section"], fontSize=32, leading=36, spaceBefore=20, spaceAfter=20))
    story = [Paragraph("SPONSOR HIGHLIGHTS  //  ACORN", styles["Caption"]),
             Paragraph("One highlight,<br/>from capture to report", styles["Hero"]),
             Paragraph("Demo walkthrough without Vergil", styles["Section"]),
             Paragraph("Current commands and reviewer controls | October 25, 2026", styles["Body"]),
             Paragraph("An eight-minute presentation using one captured TEST ONLY submission, local review, explicit approval and a local PDF. The screenshot appendix preserves dated cloud evidence.", styles["Body"]),
             Spacer(1, 16)]
    for number, title, detail in [
        ("01", "Capture", "PowerApp receipt and immutable GitLab JSONL commit"),
        ("02", "Review and apply", "Inspect one proposed entry, then confirm the local change"),
        ("03", "Graduate", "Select the contributor promotion and confirm it separately"),
        ("04", "Report", "Open the local PDF and demonstrate safe replay"),
    ]:
        story += [Paragraph(number + " // " + title, styles["Section"]), Paragraph(detail, styles["Body"])]
    story += [Spacer(1, 15), Paragraph("The following workflow map is retained from the user guide. The confirmed new-save Teams notice is automatic. Other review/report messages use the owner-triggered callback flow.", styles["Caption"]), PageBreak()]
    lines = SOURCE.read_text().splitlines()
    i = 1
    while i < len(lines):
        line = lines[i]
        if not line.strip():
            i += 1
        elif line.startswith("## "):
            story.append(Paragraph(inline(line[3:]), styles["Section"]))
            i += 1
        elif line.startswith("```"):
            block = []
            i += 1
            while i < len(lines) and not lines[i].startswith("```"):
                block += textwrap.wrap(lines[i], width=100, subsequent_indent="  ", replace_whitespace=False,
                                       drop_whitespace=False) or [""]
                i += 1
            story.append(Paragraph("<br/>".join(html.escape(value).replace(" ", "&#160;") for value in block), styles["DemoCode"]))
            i += 1
        elif line.startswith("|"):
            rows = []
            while i < len(lines) and lines[i].startswith("|"):
                cells = [cell.strip() for cell in lines[i].strip("|").split("|")]
                if not all(re.fullmatch(r"[-: ]+", cell) for cell in cells):
                    rows.append([Paragraph(inline(cell), styles["Cell"]) for cell in cells])
                i += 1
            widths = [66, 222, 216] if len(rows[0]) == 3 else [145, 359]
            table = Table(rows, colWidths=widths, repeatRows=1, hAlign="LEFT")
            table.setStyle(TableStyle([("BACKGROUND", (0, 0), (-1, 0), PALE),
                                       ("ROWBACKGROUNDS", (0, 1), (-1, -1), [colors.white, colors.HexColor("#F6F8F8")]),
                                       ("VALIGN", (0, 0), (-1, -1), "TOP"),
                                       ("LEFTPADDING", (0, 0), (-1, -1), 8), ("RIGHTPADDING", (0, 0), (-1, -1), 8),
                                       ("TOPPADDING", (0, 0), (-1, -1), 5), ("BOTTOMPADDING", (0, 0), (-1, -1), 5)]))
            story += [table, Spacer(1, 10)]
        elif line.startswith("- "):
            story.append(Paragraph("• " + inline(line[2:]), styles["Body"]))
            i += 1
        else:
            paragraph = [line]
            i += 1
            while i < len(lines) and lines[i].strip() and not lines[i].startswith(("## ", "```", "|", "- ")):
                paragraph.append(lines[i])
                i += 1
            story.append(Paragraph(inline(" ".join(paragraph)), styles["Body"]))
    for title, images in [
        ("Captured form and receipt", [("form-prepared.png", "Prepared TEST ONLY personnel highlight."),
                                       ("form-receipt.png", "Saved receipt: capture is confirmed; content review follows.")]),
        ("Captured append and replay", [("gitlab-append.png", "GitLab commit: exactly one added JSONL line."),
                                       ("cloud-replay.png", "Exact replay: the prior immutable commit is reused.")]),
        ("Local report result", [("report-page.png", "Local TEST ONLY report. The retained entry is archived; this is not remote publication.")]),
    ]:
        story += [PageBreak(), Paragraph(title, styles["Section"]),
                  Paragraph("Historical evidence captured October 24, 2026. UI labels or paths in these screens may differ from current commands.", styles["Caption"])]
        for filename, caption in images:
            picture = Image(str(ASSETS / filename))
            scale = min(WIDTH / picture.imageWidth, (242 if len(images) > 1 else 550) / picture.imageHeight)
            picture.drawWidth = picture.imageWidth * scale
            picture.drawHeight = picture.imageHeight * scale
            story += [KeepTogether([picture, Spacer(1, 5), Paragraph(caption, styles["Caption"])]), Spacer(1, 10)]
    buffer = BytesIO()
    doc = SimpleDocTemplate(buffer, pagesize=(612, 792), leftMargin=54, rightMargin=54,
                            topMargin=54, bottomMargin=51, title="Sponsor Highlights - Demo Walkthrough",
                            author="ACORN / ORNL pilot", subject="Recorded cloud capture, live local review and reporting")
    doc.build(story, onFirstPage=page, onLaterPages=page)
    reader = PdfReader(buffer)
    guide = PdfReader(ASSETS.parents[1] / "powerapps-jsonl-intake/_assets/sponsor-highlights-user-reviewer-guide.pdf")
    pages = [reader.pages[0], guide.pages[1], *reader.pages[1:]]
    writer = PdfWriter()
    for number, current in enumerate(pages, 1):
        width, height = float(current.mediabox.width), float(current.mediabox.height)
        overlay = BytesIO()
        footer = Canvas(overlay, pagesize=(width, height))
        footer.setFillColor(colors.white)
        footer.rect(0, 0, width, 45, fill=1, stroke=0)
        footer.setFillColor(INK)
        footer.setFont("Helvetica", 8)
        footer.drawString(54, 28, "ACORN / ORNL PILOT  |  TEST ONLY  |  OCT 25, 2026")
        footer.drawRightString(width - 54, 28, f"{number} / {len(pages)}")
        footer.save()
        current.merge_page(PdfReader(overlay).pages[0])
        writer.add_page(current)
    writer.add_metadata({"/Title": "Sponsor Highlights - Demo Walkthrough", "/Author": "ACORN / ORNL pilot"})
    with OUTPUT.open("wb") as output:
        writer.write(output)


def page(canvas, doc):
    canvas.saveState()
    canvas.setStrokeColor(colors.HexColor("#DDEBE7"))
    for offset in range(0, 140, 16):
        canvas.arc(480-offset, 676-offset, 660+offset, 850+offset, 160, 115)
    canvas.setStrokeColor(GREEN)
    canvas.setLineWidth(2)
    canvas.line(54, 755, 558, 755)
    canvas.restoreState()


if __name__ == "__main__":
    main()
