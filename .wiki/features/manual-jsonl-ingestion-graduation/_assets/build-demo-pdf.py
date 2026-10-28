#!/usr/bin/env python3
"""Render the current demo walkthrough and retained screenshots as a branded PDF."""
import html
from io import BytesIO
from os.path import relpath
from pathlib import Path
import re
import sys

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


sys.path.insert(0, str(ASSETS.parents[1] / 'powerapps-jsonl-intake/_assets'))
from pdf_code import shell_block


def inline(value):
    escaped = html.escape(value)
    def link(match):
        label, target = match.groups()
        if "://" not in target:
            path, _, anchor = html.unescape(target).partition("#")
            target = Path(relpath((SOURCE.parent / path).resolve(), OUTPUT.parent)).as_posix() + ("#" + anchor if anchor else "")
        return f'<a href="{html.escape(target, quote=True)}" color="#00785B">{label}</a>'
    escaped = re.sub(r"\[([^\]]+)\]\(([^)]+)\)", link, escaped)
    escaped = re.sub(r"`([^`]+)`", r'<font name="Courier" size="8.5">\1</font>', escaped)
    return re.sub(r"\*\*([^*]+)\*\*", r"<b>\1</b>", escaped)


def append_markdown(story, styles, lines):
    i = 1
    while i < len(lines):
        line = lines[i]
        if not line.strip():
            i += 1
        elif line.startswith("## "):
            if line in {"## SharePoint download to a combined local PDF", "## Appendix A — Retained GitLab rehearsal"}:
                story.append(PageBreak())
            story.append(Paragraph(inline(line[3:]), styles["Section"]))
            i += 1
        elif line.startswith("```"):
            block = []
            i += 1
            while i < len(lines) and not lines[i].startswith("```"):
                block.append(lines[i])
                i += 1
            story.append(KeepTogether([shell_block("\n".join(block), WIDTH, size=9, minimum=8.5)]))
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
            style = styles["Step"] if re.match(r"^\*\*\d+\.", paragraph[0]) else styles["Body"]
            story.append(Paragraph(inline(" ".join(paragraph)), style))


def main():
    styles = getSampleStyleSheet()
    styles.add(ParagraphStyle("Body", fontName="Helvetica", fontSize=10, leading=14.5, textColor=INK, spaceAfter=8))
    styles.add(ParagraphStyle("Section", parent=styles["Body"], fontName="Helvetica-Bold", fontSize=16, leading=20,
                              textColor=GREEN, spaceBefore=14, spaceAfter=9, keepWithNext=True))
    styles.add(ParagraphStyle("Cell", parent=styles["Body"], fontSize=8.5, leading=11.5, spaceAfter=0))
    styles.add(ParagraphStyle("Step", parent=styles["Body"], keepWithNext=True))
    styles.add(ParagraphStyle("Caption", parent=styles["Body"], fontSize=8, leading=11, textColor=colors.HexColor("#526A70")))
    styles.add(ParagraphStyle("Hero", parent=styles["Section"], fontSize=32, leading=36, spaceBefore=20, spaceAfter=20))
    story = [Paragraph("SPONSOR HIGHLIGHTS  //  ACORN", styles["Caption"]),
             Paragraph("One highlight,<br/>from capture to report", styles["Hero"]),
             Paragraph("Demo walkthrough without Vergil", styles["Section"]),
             Paragraph("Current commands and reviewer controls | Sanitized copy", styles["Body"]),
             Paragraph("Stage, inspect, apply and report using the three-row SharePoint demo. A separate eight-minute GitLab rehearsal also demonstrates optional graduation. All entries are TEST ONLY; screenshots preserve dated cloud evidence.", styles["Body"]),
             Spacer(1, 16)]
    for number, title, detail in [
        ("01", "Stage", "Capture exact receipt rows with required correction ancestry"),
        ("02", "Inspect", "Read the proposed changes and verify the selected destination"),
        ("03", "Apply", "Confirm the reviewed entries with the Yes / No buttons"),
        ("04", "Report", "Generate one combined local PDF; graduation is optional"),
    ]:
        story += [Paragraph(number + " // " + title, styles["Section"]), Paragraph(detail, styles["Body"])]
    story += [Spacer(1, 15), Paragraph("The following map shows the active SharePoint capture route. New-save Teams notices are automatic and best effort. Later review/report messages use the owner-triggered callback. The retained full rehearsal uses the GitLab fixture.", styles["Caption"]), PageBreak(),
              Paragraph("Demo scope and evidence", styles["Section"])]
    source_text = SOURCE.read_text()
    current_text, appendix_text = source_text.split("## Appendix A — Retained GitLab rehearsal", 1)
    lines = current_text.splitlines()
    append_markdown(story, styles, lines)
    story += [PageBreak(), Paragraph("SharePoint: verified receipt and export", styles["Section"]),
              Paragraph("October 26, 2026: owner-context published app created a list receipt and verified its JSONL export. The Teams action succeeded. The separate three-row owner-context download, stage, inspect, apply and report demonstration also passed October 26. Standard-license participant access, exporter boundary/repair acceptance, native restore and published correction acceptance passed October 27. This screenshot remains a dated original-save example.", styles["Body"])]
    sp_image = Image(str(ASSETS.parents[1] / "powerapps-jsonl-intake/_assets/sharepoint/app-teams-success.png"))
    sp_scale = min(WIDTH / sp_image.imageWidth, 510 / sp_image.imageHeight)
    sp_image.drawWidth = sp_image.imageWidth * sp_scale
    sp_image.drawHeight = sp_image.imageHeight * sp_scale
    story += [sp_image, Spacer(1, 12), Paragraph("Receipt 64b836e0-016f-4493-9894-a6fe79acc8de. One file per receipt; no GitLab append is performed by this branch.", styles["Caption"])]
    lines = ("# Appendix\n## Appendix A — Retained GitLab rehearsal" + appendix_text).splitlines()
    append_markdown(story, styles, lines)
    for title, images in [
        ("Appendix A: captured GitLab form and receipt", [("form-prepared.png", "Prepared TEST ONLY personnel highlight."),
                                       ("form-receipt.png", "Saved receipt: capture is confirmed; content review follows.")]),
        ("Appendix A: captured append and replay", [("gitlab-append.png", "GitLab commit: exactly one added JSONL line."),
                                       ("cloud-replay.png", "Exact replay: the prior immutable commit is reused.")]),
        ("Appendix A: historical local report", [("report-page.png", "Local TEST ONLY report. The retained entry is archived; this is not remote publication.")]),
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
    workflow = next(current for current in guide.pages if "From highlight to reviewed report" in current.extract_text())
    pages = [reader.pages[0], workflow, *reader.pages[1:]]
    writer = PdfWriter()
    for number, current in enumerate(pages, 1):
        width, height = float(current.mediabox.width), float(current.mediabox.height)
        overlay = BytesIO()
        footer = Canvas(overlay, pagesize=(width, height))
        footer.setFillColor(colors.white)
        footer.rect(0, 0, width, 45, fill=1, stroke=0)
        footer.setFillColor(INK)
        footer.setFont("Helvetica", 8)
        footer.drawString(54, 28, "ACORN / ORNL PILOT  |  TEST ONLY  |  SANITIZED COPY")
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
