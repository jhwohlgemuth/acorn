#!/usr/bin/env python3
"""Build the two current guides from retained evidence."""
import argparse
import html
import os
import re
import shutil
import subprocess
import tempfile
from pathlib import Path

from pdf_code import shell_block
from reportlab.lib.colors import HexColor, white
from reportlab.lib.styles import ParagraphStyle
from reportlab.lib.utils import ImageReader
from reportlab.pdfgen.canvas import Canvas
from reportlab.platypus import Paragraph, Table, TableStyle

ASSETS = Path(__file__).resolve().parent
RUNTIME = Path.home() / '.cache/codex-runtimes/codex-primary-runtime/dependencies'
GREEN, INK, PALE = '#00785B', '#163C47', '#EAF3F1'
ENV = '221f44a6-1468-ea4e-b93d-cd52f1723079'
GL_APP = f'https://apps.gov.powerapps.us/play/e/{ENV}/a/15ae605b-585b-49f5-a413-2b091f919a35'
SP_APP = f'https://apps.gov.powerapps.us/play/e/{ENV}/a/163f110f-7f8a-401c-894e-0786b2c018a4'
SP_SITE = 'https://ornl.sharepoint.com/sites/ResearchEnablement'
REPO = 'https://code.ornl.gov/research-enablement/experiments/power-automate'
DOCS = 'https://acorn.ornl.gov/cli-reference/commands/intake.html'
W, H = 1080, 810


def paragraph(c, text, x, top, width=972, size=17, color=INK, bold=False):
    style = ParagraphStyle('copy', fontName='Helvetica-Bold' if bold else 'Helvetica', fontSize=size,
                           leading=size*1.4, textColor=HexColor(color))
    p = Paragraph(text, style)
    _, height = p.wrap(width, H)
    if top + height > 748:
        raise ValueError(f'Content exceeds page: {text[:80]}')
    p.drawOn(c, x, H-top-height)
    return top+height+14


def picture(c, path, x, top, width, height):
    img = ImageReader(str(path))
    iw, ih = img.getSize()
    scale = min(width/iw, height/ih)
    dw, dh = iw*scale, ih*scale
    c.drawImage(img, x+(width-dw)/2, H-top-dh, dw, dh, mask='auto')
    return top+dh


def grid(c, rows, top, widths=(200, 386, 386), size=15):
    style = ParagraphStyle('cell', fontName='Helvetica', fontSize=size, leading=size*1.35, textColor=HexColor(INK))
    data = [[Paragraph(str(value), style) for value in row] for row in rows]
    t = Table(data, colWidths=widths)
    t.setStyle(TableStyle([('BACKGROUND',(0,0),(-1,0),HexColor('#DCEDE7')),
                          ('ROWBACKGROUNDS',(0,1),(-1,-1),[white,HexColor('#F3F6F7')]),
                          ('VALIGN',(0,0),(-1,-1),'TOP'),('LEFTPADDING',(0,0),(-1,-1),12),
                          ('RIGHTPADDING',(0,0),(-1,-1),12),('TOPPADDING',(0,0),(-1,-1),10),
                          ('BOTTOMPADDING',(0,0),(-1,-1),10)]))
    _, height = t.wrap(972, H)
    if top+height > 744:
        raise ValueError('Table exceeds page')
    t.drawOn(c,54,H-top-height)
    return top+height+18


def code(c, text, top, size=14):
    block = shell_block(text, 944, size, minimum=11)
    _, height = block.wrap(944, H)
    if top + height + 16 > 748:
        raise ValueError('Code exceeds page')
    block.drawOn(c, 68, H-top-height-8)
    return top+height+30


def header(c, title, section, number, total, audience):
    c.setFillColor(HexColor('#004C54')); c.rect(0,H-132,W,132,fill=1,stroke=0)
    c.saveState()
    clip=c.beginPath();clip.rect(0,H-132,W,132);c.clipPath(clip,stroke=0)
    c.setStrokeColor(HexColor('#296C6C')); c.setLineWidth(0.6)
    for d in range(0,170,19):
        c.arc(860-d,655-d,1150+d,955+d,165,110)
    c.restoreState()
    c.setFillColor(HexColor('#86E3BB')); c.rect(54,H-103,4,62,fill=1,stroke=0)
    paragraph(c,section.upper(),73,29,910,12,'#86E3BB',True)
    paragraph(c,title,73,58,920,29,'#FFFFFF',True)
    c.setStrokeColor(HexColor('#C4D9DB'));c.line(54,47,1026,47)
    c.setFillColor(HexColor('#526A70'));c.setFont('Helvetica',10)
    c.drawString(54,29,f'ACORN / ORNL PILOT  //  {audience}  //  OCT 27, 2026  //  SANITIZED')
    c.drawRightString(1026,29,f'{number:02} / {total:02}')


def link(label, url):
    return f'<a href="{html.escape(url,quote=True)}" color="{GREEN}"><b>{label}</b></a>'


def diagram(c):
    paragraph(c,'New highlights and later corrections follow the same SharePoint review and reporting path.',54,153,size=17)
    picture(c,ASSETS/'notifications/workflow.png',85,192,910,465)
    paragraph(c,'Solid green: saved data and review steps. A newly exported original or correction triggers Teams, best effort. Dashed: owner-triggered milestones. Corrections do not alter existing reports; local CLI commands send no messages.',54,676,size=14)


def branch_overview(c):
    paragraph(c,'Use the SharePoint app for new highlights. The separate GitLab app is retained with both flows Off.',54,157)
    top=grid(c,[['','SharePoint pilot','GitLab connector'],
      ['Open',link('Sponsor Highlights - SharePoint Pilot',SP_APP),link('Retained app - flows Off',GL_APP)],
      ['Storage','One shared list row per receipt. Combined logbook.jsonl refreshes every five minutes; individual receipt files remain available.','One appended line in the repository JSONL logbook; receipt identifies a commit.'],
      ['Projects','13 seeded SharePoint catalog rows. Updates are maintained explicitly.','Repository configuration: .acorn.jsonc folders, or index.json / index.yaml discovery.'],
      ['Current scope','Published with accepted recovery/export repairs. Standard-license participant access passed; approved audience remains owner and participant.','Paused October 27. Custom connector and credentials retained. Restoration requires premium use rights; intermittent denials remain.'],
      ['After capture','Download combined logbook.jsonl for local ACORN review. Allow the next five-minute export run.','Stage the GitLab file URL or an exact local snapshot for ACORN review.'] ],198,size=15)
    paragraph(c,'Keep pending GitLab receipts in the original app. Ask the owner to reconcile them; never resubmit the same uncertain highlight through SharePoint.',54,top,size=16,bold=True)


def start(c):
    top=paragraph(c,'Use Sponsor Highlights - SharePoint Pilot to submit highlights and correct your own confirmed submissions. Each save retains an immutable receipt for review.',54,156)
    top=paragraph(c,link('Open the SharePoint form',SP_APP),54,top,size=21)
    top=grid(c,[['Step','What happens'],['1 // Submit or correct','For new work, complete the form and submit once. For a correction, use My Submissions > Edit selected > Save correction.'],['2 // Confirm','JSONL ready confirms the individual receipt file. Teams notifies the owner and submitter; self-submissions skip the duplicate notice.'],['3 // Collect','The five-minute exporter rebuilds logbook.jsonl from the stored list receipts. Operators check freshness before downloading.'],['4 // Review and report','Stage the downloaded rows, inspect the latest content, confirm apply and generate a new local report. Email is reserved for important milestones.']],top,widths=(220,752),size=17)
    paragraph(c,'This guide follows the active SharePoint workflow. Appendix A preserves the paused GitLab connector reference. Retry each pending save with its same receipt ID. A correction gets a new linked ID; original records remain unchanged.',54,top,size=16)


def first_load(c):
    paragraph(c,'On first load, Power Apps may ask: <b>Allow Sponsor Highlights - SharePoint Pilot to access your data?</b> This is the expected connection-consent step before using the form.',54,156,size=17)
    popup=ImageReader(str(ASSETS/'sharepoint/first-load/connection-consent-redacted.png'))
    popup_width,popup_height=popup.getSize()
    height=590*popup_height/popup_width
    c.saveState()
    c.setFillColor(HexColor('#163C47'))
    c.setFillAlpha(0.012)
    for layer in range(12,0,-1):
        spread=layer*0.6
        c.roundRect(54-spread,H-222-height-4-spread,590+2*spread,height+2*spread,spread+2,fill=1,stroke=0)
    c.restoreState()
    picture(c,ASSETS/'sharepoint/first-load/connection-consent-redacted.png',54,222,590,355)
    paragraph(c,'First-load dialog. Account addresses removed; surrounding browser/profile details cropped out.',54,582,590,size=11)
    top=paragraph(c,'Why these connections are needed',675,222,351,size=19,bold=True)
    top=paragraph(c,'<b>SharePoint</b> supports the project catalog, saved highlights and corrections, and JSONL receipt/export files.',675,top,351,size=16)
    top=paragraph(c,'<b>Microsoft Teams</b> supports save confirmations to the submitter and operator through Power Automate.',675,top,351,size=16)
    paragraph(c,'The two services shown match this app\'s documented workflow. Use your ORNL work account if Microsoft asks you to sign in.',675,top,351,size=16)
    top=paragraph(c,'<b>What to do:</b> open the form from the trusted ORNL link in this guide, check the app name and the two services shown, then select <b>Allow</b>. Wait for the form to finish loading.',54,625,size=16)
    paragraph(c,'Allow authorizes the displayed connections; it does not submit a highlight. If the app name or services differ, or a connection reports an error, ask the app owner before continuing.',54,top,size=14)
    paragraph(c,'Microsoft reference: '+link('Canvas-app connection consent','https://learn.microsoft.com/en-us/power-apps/maker/canvas-apps/connections-list#connection-consent-dialog')+'.',54,724,size=11)


def projects(c):
    paragraph(c,'Report placement is first and defaults to Project highlight. Check Submitted by, then load the project list. My Submissions sits between the banner and form for later corrections.',54,156)
    paragraph(c,'The catalog may load automatically. The button reads <b>Load Project List</b> before its first successful load and <b>Refresh Project List</b> afterward. Loading alone does not submit anything.',54,218,size=16)
    picture(c,ASSETS/'sharepoint/submission-editing/navigation-published.png',54,290,972,300)
    paragraph(c,'The catalog has 13 project choices. Report-wide update (no associated project) disables Project and its load button. For project highlights, failed catalog loading blocks submission and offers Retry Project List.',54,658,size=14)


def fields(c):
    top=grid(c,[['Field','What to enter'],['Report placement','Project highlight is the default. Report-wide update (no associated project) routes to the separate general logbook; standalone CLI reporting remains PA-29.'],['Project / date','For project highlights, load and select the research project. Enter the date of the work.'],
     ['Category','administration, collaboration, data, engagement, funding, infrastructure, method, other, personnel, publication, research or software.'],
     ['Status','completed, in-progress or upcoming.'],['Event type','Optional: conference, exhibition, meeting, other, presentation, tour, training or workshop.'],
     ['Summary / details','Summary: required, up to 1,200 characters. Details: optional, one per line; up to 20 details, each up to 1,000 characters.']],156,widths=(215,757),size=16)
    paragraph(c,'The form captures the original fields in one submission envelope. Include a concrete result and why it matters. Use non-sensitive content because pending submissions are stored in the browser.',54,top,size=17)


def write_highlight(c):
    top=paragraph(c,'Review the project, date, category, status and text. Select <b>Submit highlight</b> once. The app freezes the payload and receipt ID while it waits for confirmation.',54,156)
    top=grid(c,[['Check','Before you send'],['Project','For a project highlight, load and choose the correct project. Report-wide updates disable these controls. Never substitute an unrelated project.'],['Summary and details','Describe the result and why it matters. Use only non-sensitive content: the pending draft is stored locally in your browser.'],['Placement','Project highlight is the default. Use Report-wide update (no associated project) only for program- or organization-wide work. It does not publish; standalone general reporting is pending.'],['After Submit','Wait for the result panel. Keep the same receipt if the result is uncertain; do not start another highlight for the same work.']],top,widths=(240,732),size=17)
    paragraph(c,'A confirmed submission is ready for operator review. The next page shows the published SharePoint confirmation and the distinction between an individual receipt and the collected logbook.',54,top,size=17)

def field_menus(c):
    paragraph(c,'Set the date of the work, then choose the category that best describes it.',54,156)
    picture(c,ASSETS/'walkthrough/07-date-picker.png',54,231,474,390)
    picture(c,ASSETS/'walkthrough/03-categories.png',552,231,474,390)
    paragraph(c,'Dated form captures, October 24. Left: calendar picker; confirm with Ok or use Cancel to retain the prior date. Right: category menu; scroll for additional choices. Earlier project labels in the captures are not the current catalog inventory.',54,661,size=15)


def status_menus(c):
    paragraph(c,'Status is required. Event type can remain Not specified if the highlight has no associated event. Choose the most specific applicable value.',54,156)
    picture(c,ASSETS/'walkthrough/05-status-options.png',54,231,474,390)
    picture(c,ASSETS/'walkthrough/06-event-options.png',552,231,474,390)
    paragraph(c,'Dated form captures, October 24. Left: progress status. Right: optional event type. The form remains an intake surface; choosing Completed describes the work, not reviewer approval.',54,661,size=15)


def recovery_panels(c):
    paragraph(c,'An unresolved save and a conflicting receipt require different actions. Read the result before retrying or editing.',54,156)
    picture(c,ASSETS/'sharepoint/pa21-20261026/unconfirmed.png',54,230,474,390)
    picture(c,ASSETS/'sharepoint/pa21-20261026/conflict-panel.png',552,230,474,390)
    paragraph(c,'Isolated SharePoint PA21 acceptance captures: unconfirmed outcome (left) and conflict (right). Preserve the same receipt and original content for retry or owner investigation; never clear an unresolved draft.',54,661,size=15)


def mobile(c):
    paragraph(c,'The form stacks vertically. My Submissions occupies its own row between the banner and form; it no longer overlaps the header.',54,156)
    picture(c,ASSETS/'sharepoint/submission-editing/navigation-phone.png',54,232,350,420)
    top=paragraph(c,'Report placement comes first. Project highlight is the default; load a project only for that choice.',450,248,550,size=19)
    top=paragraph(c,'For an existing confirmed receipt, use My Submissions to load your own history and start a linked correction. Pending submissions must reconcile before another edit.',450,top,550,size=18)
    paragraph(c,'October 27 phone-size Studio preview. This verifies spacing, not native-mobile storage acceptance. Keep one tab per pending browser slot.',450,top,550,size=16)


def sp_receipt(c):
    paragraph(c,'<b>Highlight saved · JSONL ready</b> means the list receipt exists and the exported file passed exact readback. Keep the receipt ID. Saving is separate from content approval and publication.',54,156)
    picture(c,ASSETS/'sharepoint/cutover/published-submission.png',54,218,972,447)
    paragraph(c,'October 27 deployed app: receipt 807ecb08-3f88-4aa8-a076-f623680b56db. Individual download matched the eighth collected row exactly; the prior seven rows were unchanged. Teams action succeeded.',54,681,size=14)


def gl_receipt(c):
    paragraph(c,'<b>Highlight saved</b> confirms the exact submission in the GitLab logbook at the displayed commit. Keep both the receipt ID and commit reference; a catalog load alone is not a saved highlight.',54,156)
    picture(c,ASSETS/'notifications/form-confirmed.png',54,220,972,447)
    paragraph(c,'Dated published acceptance, October 24: receipt 82a8dd58-8c60-4ec5-abf5-cb251ab9e033 at commit 03ff58362894. Later GitLab network denials remain a known limitation.',54,681,size=14)


def recovery(c):
    top=grid(c,[['Result','SharePoint action'],['Confirmed','Receipt plus verified export. Back to highlight preserves the locked entry; My Submissions starts a separate correction.'],['Received; export pending','Keep the same ID. Use Retry JSONL export; the list receipt alone is not an export confirmation.'],['Uncertain save','Keep the frozen content and retry the existing submission.'],['Rejected / conflict','Edit only after explicit rejection permits it. Conflicting ID/content needs owner review.'],['Browser recovery unavailable','A new send is stopped if pending storage cannot be prepared. Do not clear existing browser data while a result is unresolved.']],156,widths=(265,707),size=17)
    paragraph(c,'Use one tab and the same browser profile/account. New highlight clears only this app\'s recovery slot; it does not delete the stored receipt. Restart, denied-storage, pending-export and conflict acceptance passed in isolated copies. The deployed update preserves its recovery key and formulas.',54,top,size=16)

def teams_mock(c):
    paragraph(c,'Fictional examples show the new message hierarchy. Teams clients may simplify colors and borders.',54,155,size=16)
    c.setFillColor(HexColor('#484772'));c.roundRect(54,147,972,449,10,fill=1,stroke=0)
    paragraph(c,'Teams  //  Sponsor Highlights',74,227,920,16,'#FFFFFF',True)
    for x,branch,target in [(74,'OWNER NOTICE','Open JSONL export'),(554,'CONTRIBUTOR CONFIRMATION','Keep this receipt for support')]:
        c.setFillColor(white);c.roundRect(x,170,452,368,8,fill=1,stroke=0)
        paragraph(c,branch,x+22,284,408,10,'#526A70',True)
        paragraph(c,'ACORN // SPONSOR HIGHLIGHTS',x+22,310,408,11,'#526A70',True)
        paragraph(c,'Highlight received',x+22,338,408,23,GREEN,True)
        c.setStrokeColor(HexColor(GREEN));c.setLineWidth(4);c.line(x+24,H-391,x+24,H-544)
        y=paragraph(c,'<b>Project</b><br/>Example Project',x+40,385,385,14)
        y=paragraph(c,'<b>Status</b><br/>Saved · JSONL verified · Awaiting review',x+40,y,385,14)
        paragraph(c,'<b>Receipt ID</b><br/>EXAMPLE-001',x+40,y,385,12)
        paragraph(c,target,x+22,553,408,14,GREEN,True)
        c.setStrokeColor(HexColor('#C4D9DB'));c.setLineWidth(0.6);c.line(x+22,H-588,x+430,H-588)
        paragraph(c,'<i>Saving does not approve or publish the highlight.</i>',x+22,596,408,11,'#526A70')
    paragraph(c,'Bold labels and indented details remain useful when a client omits styling. Owner notices link to stored JSONL; contributors receive their receipt without a private-storage link. Email stays reserved for milestones.',54,678,size=14)


def notifications(c):
    top=grid(c,[['Milestone','Channel','When it happens today'],
      ['Original or correction + export','Teams to owner + submitter','Automatic after a new receipt and verified export in the same run. History reads and exact replays send no notice. Skip duplicate owner self-notices.'],
      ['Workspace created / no report changes','Teams to owner','Owner invokes the manual callback for an actual milestone.'],
      ['Review requested / report ready / actionable failure','Email to owner','Owner invokes the manual callback with the real review or deliverable evidence.'],
      ['Typing, validation or unconfirmed save','App feedback only','No routine intake email; no new notice for exact replays.']],156,size=15)
    paragraph(c,'SharePoint notification limitation: an export that first succeeds on a later retry does not send the initial new-receipt notice. Delivery is best effort. Local CLI commands do not invoke these callbacks; Vergil automation remains separate work.',54,top,size=16)


def reviewer(c):
    top=paragraph(c,'The form collects submissions; it does not provide an approval screen. The owner supplies the proposed changes and review location.',54,156)
    for title,body in [('1 // Check the evidence','Match the project and receipt. Review date, category, status, placement, summary and details against the underlying work.'),('2 // Inspect the local proposal','Stage the downloaded originals and corrections together. Inspect the latest content and linked history, then confirm apply. Conflicts require operator review.'),('3 // Confirm optional promotion','Graduation may add a contributor or other supported metadata. Use the interactive run --index mode for per-entry Yes / No selection and a separate final confirmation.'),('4 // Review the report','A local PDF is a preview, not publication. Follow the supplied review process before merging changes or releasing a report.')]:
        top=paragraph(c,title,54,top,size=20,bold=True)
        top=paragraph(c,body,54,top,size=17)
    paragraph(c,'Use My Submissions to correct your own confirmed highlight. Operators review the new linked receipt. Never overwrite stored payloads or create duplicates just to retry reporting.',54,top,size=16,color=GREEN,bold=True)


def correction_history(c):
    paragraph(c,'An edit adds a linked receipt. It preserves the original and the complete correction history.',54,153,size=17)
    picture(c,ASSETS/'notifications/correction-history.png',70,203,940,400)
    top=paragraph(c,'Example IDs A, B and C represent three saves of one highlight. The download keeps all three rows; reviewed apply keeps one current logbook entry with the latest accepted content.',54,623,size=15)
    paragraph(c,'Keep ancestry in the batch or previously applied workspace history. Missing parents, competing corrections or changed/graduated prior content stop apply for review. Existing PDFs stay unchanged; generate and review a new report.',54,top,size=14)


def editing(c):
    top=paragraph(c,'Use My Submissions between the banner and form to correct a confirmed highlight. You can see only your own receipt history.',54,156)
    top=grid(c,[['Step','Action'],['1 // Choose','Open My Submissions, select the latest receipt and choose Edit selected.'],['2 // Correct','Update date, category, status, event type, summary or details. Project and report placement stay fixed. Cancel editing leaves the saved record unchanged.'],['3 // Save once','Select Save correction. Wait for confirmed JSONL feedback and keep the new receipt ID. If uncertain, retry the same frozen correction.'],['4 // Review','The new receipt links the original and previous version. Both stay in collected JSONL; operators stage, inspect and apply the latest version before reporting.']],top,widths=(210,762),size=16)
    top=paragraph(c,'A correction sends Teams confirmation after a newly saved, verified export. It does not approve the highlight, reverse graduation or change an existing report. Email remains milestone-only.',54,top,size=16)
    paragraph(c,'Published and accepted October 27 for the owner and a standard-license contributor, including private history and Teams confirmation. If another correction won first, reload history before editing again.',54,top,size=14,color=GREEN)


def support(c):
    top=paragraph(c,'If access fails, keep the exact error and app link. A standard connector does not grant an account entitlement, app sharing, site access or environment permission.',54,156)
    top=grid(c,[['Check','Current position'],['SharePoint participant access','Standard-license participant access passed October 27. Current approved audience: owner and participant. Wider group rollout remains separate.'],['Help request','Send the owner the app/branch, receipt ID, selected project, approximate time and visible result. Never send a token or API key.'],['Pending submission','Keep its original app, ID and content. Do not discard an unresolved receipt or resubmit it under a new ID.']],top,widths=(240,732),size=16)
    paragraph(c,link('Open SharePoint pilot',SP_APP)+'  |  '+link('ACORN intake reference',DOCS),54,top,size=16)
    paragraph(c,'Microsoft references: '+link('SharePoint connector', 'https://learn.microsoft.com/en-us/connectors/sharepointonline/')+'; '+link('Power Apps license designation','https://learn.microsoft.com/en-us/power-apps/maker/canvas-apps/license-designation')+'. Pilot acceptance is recorded in sharepoint-pilot.md and the active PA plan.',54,top+57,size=13)


def components(c):
    rows=[['Component','SharePoint pilot'],['Canvas app','163f110f-7f8a-401c-894e-0786b2c018a4'],['Intake flow','64679e9c-ae76-4263-b8fa-8832835516ca'],['Catalog flow','6f8a4ab5-dd66-4904-9f5d-f307519ab4d5'],['History / correction flow','36695ee1-47d2-f111-a05c-001dd8062052'],['Collected exporter','51efa5f9-6e69-4d75-a53b-2c2965fd51bd'],['Browser slot','SponsorHighlightSharePointPending'],['Destination','Research Enablement site: Intake Pilot list and JSONL Pilot library']]
    top=grid(c,rows,156,widths=(250,722),size=17)
    top=paragraph(c,'Environment: <b>Wohlgemuth, Jason</b> ('+ENV+'). The app binds three standard SharePoint pilot flows; the collected exporter runs separately every five minutes.',54,top,size=16)
    paragraph(c,'Owner-only milestone receiver: dd1d8f67-116c-f55d-7752-7c7d322691c5. It retains Dataverse receipt tracking and is separate from the standard-only contributor bindings.',54,top,size=15)

def sp_contract(c):
    top=grid(c,[['Field / behavior','Contract'],['SubmissionId','Required, unique 36-character UUID. Find/read the receipt before confirming; changed payload under the same UUID conflicts.'],['PayloadBase64A / B','Base64 of exact original UTF-8 JSON. A holds the first 45,000 encoded characters; B holds the remainder. Encoding is not encryption.'],['Correction links','revision.rootSubmissionId identifies the original; supersedesSubmissionId identifies the previous receipt. Each correction has a new SubmissionId. Never overwrite a stored payload.'],['JSONL export','Receipt: decode A + B, add LF and verify UUID.jsonl. Combined: one serialized five-minute flow rebuilds logbook.jsonl from list rows, verifies exact bytes and skips unchanged writes.'],['ExportStatus / URL','Pending until exact readback succeeds; then Exported with the confirmed file location. A receipt alone is not a successful export.']],156,widths=(230,742),size=15)
    paragraph(c,'Correction intake verifies authenticated ownership, current predecessor and fixed project/placement. The unique, nullable, indexed CorrectionParentId prevents competing successors; exact replays reuse their receipt. History is caller-filtered and bounded. Combined export retains all receipts, caps at 5,000 rows / 10 MiB and preserves its prior file on failure. Display fields are not an export source.',54,top,size=15)


def sp_run(c):
    top=paragraph(c,'Read action outcomes, not just the overall run color. A replay can fail non-overwrite file creation while successfully reading and confirming the existing exact file.',54,156)
    top=grid(c,[['Case','Observed October 26','Run suffix'],['Fresh app + Teams','Receipt, export/readback, response and Teams action succeeded.','...94369117892624466851CU16'],['Exact replay after Teams','Receipt creation skipped; existing file matched; Teams skipped.','...92740963250031363046CU21'],['Changed payload','Conflict branch succeeded; receipt/file writes skipped.','...11980621045990695644CU06'],['Malformed envelope','Parse failed; persistence skipped; outer response succeeded.','...11189995757802689639CU25']],top,widths=(185,535,252),size=14)
    paragraph(c,'Response outputs: status, submissionid, exportstatus, exportfileurl and receipturl. The app accepts exported only with a matching UUID, received status, exported state and expected file URL. Uncertain export retains the frozen payload and offers Retry JSONL export.',54,top,size=16)
    paragraph(c,'Full run IDs and action evidence: sharepoint-pilot.md and _assets/sharepoint/. Secure history hides response values; the published app confirmed the successful response contract. Isolated app-driven failure/retry acceptance passed. October 27 live deployment also verified eight exact collected rows and Teams delivery action.',54,top+95,size=14)


def catalogs(c):
    top=grid(c,[['Contract','SharePoint catalog'],['Source','13 project choices plus a reserved general destination. Project choices were seeded from revision 03ff5836289447034f40f2a4898c155441e6cb66. No automatic repository synchronization.'],['Response','catalogjson text: source=sharepoint, projects=[{id,label}]. More than 1,000 rows returns catalog_limit with an empty list.'],['Failure feedback','Safe access_denied, unavailable, invalid_catalog or catalog_limit response. The app shows persistent guidance and Retry Project List, with submission disabled until a valid list loads.'],['Acceptance','13-project live load and isolated overflow/error/retry acceptance passed. Accepted response and app feedback are deployed.']],156,widths=(220,752),size=16)
    picture(c,ASSETS/'sharepoint/catalog-success.png',54,top,972,210)
    paragraph(c,'October 26 catalog capture. Catalog membership does not replace ACORN folder/member authorization during later admission.',54,696,size=13)

def gl_persistence(c):
    paragraph(c,'Validate the caller/envelope, resolve the project, check immutable history for an exact replay, then conditionally append one line. Preserve every previous byte and verify the committed snapshot before success.',54,156)
    picture(c,ASSETS/'published-append-commit.png',54,225,972,385)
    paragraph(c,'Dated append evidence. Exact replays return the verified prior commit without another append or Teams notice. History lookup is bounded; missing proof remains unresolved. Preserve the ID and payload across timeouts or network denials.',54,630,size=16)
    paragraph(c,'PA-02 is closed under the owner-approved known-limitation decision. That decision does not establish reliable writes or close the remaining boundary, concurrency or restore tests.',54,697,size=13)


def local_handoff(c):
    top=paragraph(c,'Download logbook.jsonl after its next five-minute refresh. Preserve exact rows and IDs. For corrections, include the original and intervening receipts, or use already-applied workspace ancestry. Stage and inspect before confirmed apply; one logical entry retains the latest accepted content.',54,156,size=15)
    top=paragraph(c,'From the prepared demo directory, use acorn on PATH. intake.json and report.json point to the same separate demo index. Replace BATCH_ID below with the value printed by stage.',54,top,size=15)
    top=code(c,'cd output/pdf/sharepoint-demo-20261026\n'
        '# 1. Stage the three exact selected rows.\n'
        'acorn --offline intake stage --batch --config intake.json \\\n  --source selected-demo.jsonl\n'
        '# 2. Inspect; 3. apply with the normal Yes / No buttons.\n'
        'acorn --offline intake inspect --config intake.json --batch-id BATCH_ID\n'
        'acorn --offline intake apply --config intake.json --batch-id BATCH_ID\n'
        '# 4. Generate the combined local PDF.\n'
        'acorn --offline report --config report.json \\\n  --from 2026-10-26 --through 2026-10-27 \\\n  --dry-run --combine',top,13)
    top=paragraph(c,'Open .acorn/reports/report.pdf. Defaults use the current directory and all configured projects (only acorn here). Combined mode needs no commit. Dates mean midnight UTC: --from is inclusive; --through is exclusive. Set CHROME_PATH only if browser discovery fails.',54,top,size=14)
    paragraph(c,'Direct SharePoint URLs remain planned; download the file first. Local reporting sends no messages or callbacks; optional graduation is separate.',54,top,size=14)


def graduation(c):
    top=paragraph(c,'After ingestion, use the current local graduation API. Supported entries can propose contributor or other metadata changes; unsupported entries remain unchanged.',54,156)
    top=code(c,'acorn intake run --index project/index.json --dry-run\nacorn intake run --index project/index.json --interactive\nacorn intake run --index project/index.json --force',top,16)
    top=grid(c,[['Mode','Meaning'],['--dry-run','Validate and preview; write nothing.'],['--interactive','Per-candidate Yes / No selection, then separate final write confirmation. No is the default.'],['--force','Accept all eligible promotions without prompts; validation, digest checks and exact backup remain.'],['--accept ID --expected-digest DIGEST','Apply only an explicitly reviewed selection with the pinned digest.']],top,widths=(340,632),size=15)
    paragraph(c,'Promotion retains and archives source entries and writes an adjacent exact-byte backup. A bare run --index with eligible entries requires an explicit approval mode. Neither local ingestion nor graduation sends milestone messages.',54,top,size=15)


def delivery(c):
    top=grid(c,[['Route','Automatic Teams gate','Recovery limit'],['SharePoint','Create_receipt_SP and Mark_exported_SP must both succeed in the same run. Runs after response handling.','Exact replay skips. Export first confirmed on a later retry also skips the initial notice; no durable notice outbox.']],156,size=15)
    top=paragraph(c,'SharePoint sends the owner a record link and the submitter a separate confirmation without that private link; self-submissions skip duplication. Secure inputs/outputs and disabled automatic send retries remain. Never resubmit only to get a message.',54,top,size=16)
    top=grid(c,[['Callback event','Required additional fields','Channel'],['mr-created / review-needed','merge_request_url, project_id','Teams / email'],['report-no-changes','project_id, through','Teams'],['artifact-ready','checksum, package_url, project_id, through','Email'],['failed','message, project_id','Email']],top,widths=(275,520,177),size=14)
    paragraph(c,'Manual callback: exact Webhook ID + Event JSON. Unique claim; frozen payload and destination; Pending > Sending > Sent; concurrency one. Sent replay skips. Uncertain Sending holds for reconciliation against the recorded attempt. Never reset solely because a send timed out.',54,top,size=14)


def operations(c):
    top=grid(c,[['Area','Current acceptance and operating action'],['SharePoint default','PA-19 through PA-28 SharePoint work is accepted. Published app keeps its ID, audience, submission formulas and recovery key. Eight-row collected download preserves the prior seven rows exactly.'],['Rollback SharePoint','Dated sanitized native copies in cutover/ and backup/ predate correction controls. Keep current editing sources separately. Restore approved settings and revalidate an isolated import; historical hashes describe originals. Never blindly overwrite an uncertain write.'],['Editing acceptance','Published editing passed owner and standard-license contributor checks: private history, correction save and Teams confirmation. The 18-row aggregate preserves original and correction bytes; local review/apply also passed.'],['Remaining rollout','Pilot stays in Wohlgemuth, Jason for owner and participant. PA-12 owns shared environment, groups and notification migration. Monitor exporter history and file freshness; no automatic exporter-failure alert is implemented.']],156,widths=(245,727),size=14)
    paragraph(c,'Keep pending receipts in their original browser slot and reconcile the same UUID. Never create a duplicate to work around uncertainty. Teams/email are notifications; exact stored receipts and reviewed indexes are the evidence.',54,top,size=15)


def references(c):
    top=paragraph(c,'Working links',54,156,size=21,bold=True)
    for label,url in [('SharePoint pilot app',SP_APP),('Research Enablement SharePoint site',SP_SITE),('ACORN intake command reference',DOCS),('Microsoft SharePoint connector classification','https://learn.microsoft.com/en-us/connectors/sharepointonline/'),('Microsoft Power Apps license designation and linked-flow caveat','https://learn.microsoft.com/en-us/power-apps/maker/canvas-apps/license-designation')]:
        top=paragraph(c,link(label,url),54,top,size=16)
    top=paragraph(c,'Repository evidence',54,top+5,size=21,bold=True)
    paragraph(c,'Within .wiki/features/powerapps-jsonl-intake/: sharepoint-pilot.md, notification-walkthrough.md, boundary-acceptance.md, restore-target-readiness.md and index.md. Manual workflow: ../manual-jsonl-ingestion-graduation/walkthrough.md and handback.md.<br/><br/>Screenshots and technical records are sanitized copies. Historical hashes describe the originals; revalidate account settings and package imports before reuse. Synthetic records remain TEST ONLY. Updated October 27 after cutover, contributor correction acceptance and navigation spacing verification. Current route, rollback evidence and native before/after app backups: _assets/sharepoint/cutover/README.md.',54,top,size=16)


def gl_submit(c):
    paragraph(c,'After the owner restores the route, open the original app, load the repository project list and complete the same highlight fields. Select Submit highlight once; the app freezes the payload and UUID until the result is reconciled.',54,156)
    picture(c,ASSETS/'notifications/form-before-submit.png',54,232,972,425)
    paragraph(c,'Historical GitLab form capture, October 24: a synthetic LandScan example. This paused route requires premium use rights. Keep a pending receipt in its original app; do not create a SharePoint duplicate.',54,681,size=14)


def gl_recovery(c):
    top=paragraph(c,'Both GitLab flows are Off. Keep any existing pending receipt in the original app and browser profile. Ask the owner to reconcile it before retrying; never copy an uncertain submission into SharePoint.',54,156)
    top=grid(c,[['Result after restoration','What to do'],['Confirmed','Keep the receipt UUID and immutable commit reference. Back to highlight preserves the locked entry.'],['Uncertain or network error','Retry the same frozen ID and payload only after the owner restores and checks the route. Missing immutable proof remains unresolved.'],['Rejected / conflict','Explicit rejection may permit correction. A conflicting ID or payload needs owner review.'],['Notifications','Only a confirmed new save sends the owner a Teams notice. Replay, rejection and unresolved results do not resend it. Missing Teams does not undo a saved commit.']],top,widths=(270,702),size=16)
    top=paragraph(c,'This custom connector requires appropriate premium use rights. Intermittent network-range 403/504 denials remain a known limitation. A successful catalog load does not prove a later write.',54,top,size=16)
    paragraph(c,link('Retained original app (paused)',GL_APP)+'  |  '+link('Pilot repository',REPO),54,top,size=16)


def gl_inventory(c):
    top=paragraph(c,'The GitLab route is preserved with intake and catalog Off. No connector, protected connection, credential, app binding or stored data was deleted during the SharePoint cutover.',54,156)
    top=grid(c,[['Component','Retained identity'],['Canvas app','15ae605b-585b-49f5-a413-2b091f919a35'],['Intake','ACORN Logbook GitLab JSONL Intake (Draft)<br/>ff4a134b-5fd3-4465-ab19-0325c2f2bf1b'],['Catalog','ACORN Project Catalog<br/>cd27888f-dd74-4675-9dd1-abc246ba56d5'],['Browser slot','SponsorHighlightPending'],['Destination','Project 22464 / main / .acorn/intake/logbook.jsonl']],top,widths=(210,762),size=15)
    top=paragraph(c,'Catalog: .acorn.jsonc folders is authoritative, including an empty list. Without it, discover folders containing index.json or index.yaml. Resolve canonical choices at one repository revision; invalid configuration or denied reads stop discovery.',54,top,size=15)
    paragraph(c,link('Retained app (paused)',GL_APP)+'  |  '+link('Pilot GitLab repository',REPO),54,top,size=16)


def gl_restore(c):
    top=paragraph(c,'Restoration normally needs two flow toggles in Wohlgemuth, Jason. Existing credentials must still be valid, premium entitlement must cover the intended users, and GitLab access must succeed.',54,156)
    for title,body in [('1 // Enable and check','Turn On ACORN Project Catalog and ACORN Logbook GitLab JSONL Intake (Draft). Verify connection health and catalog load. Retained bindings mean no reimport or new token is normally needed.'),('2 // Reconcile before new work','Keep the original UUID and exact payload for pending receipts. Check immutable history and retry the same ID. Do not resubmit uncertain GitLab content through SharePoint.'),('3 // Preserve notification semantics','Queued_new must succeed before the owner Teams notice. Replay, rejection and unresolved results skip delivery. A send failure does not roll back the commit; do not resubmit merely to get a message.')]:
        top=paragraph(c,title,54,top,size=18,bold=True)
        top=paragraph(c,body,54,top,size=15)
    top=paragraph(c,'For local review, stage a GitLab file URL online with CI_SERVER_HOST and the protected credential reference, or stage an exact local snapshot. Then use the same inspect / apply / report steps. Local commands send no milestones.',54,top,size=15)
    paragraph(c,'PA-15 boundaries, PA-16 concurrency and PA-17 full restoration checks remain independent unfinished gates. PA-02 records accepted intermittent connectivity. Full rollback evidence: _assets/sharepoint/cutover/README.md.',54,top,size=14)


def render_diagram(stem='workflow'):
    folder = ASSETS/'notifications'
    with tempfile.TemporaryDirectory(prefix='pa-guides-') as name:
        root=Path(name);(root/'src').mkdir()
        (root/'book.toml').write_text('[book]\ntitle="Workflow"\n[preprocessor.pikchr]\ncommand="'+str(Path.home()/'.cargo/bin/mdbook-pikchr')+'"\n')
        (root/'src/SUMMARY.md').write_text('# Summary\n- [Workflow](workflow.md)\n')
        (root/'src/workflow.md').write_text('```pikchr\n'+(folder/f'{stem}.pikchr').read_text()+'\n```')
        subprocess.run([shutil.which('mdbook') or str(Path.home()/'.cargo/bin/mdbook'),'build',str(root)],check=True)
        content=(root/'book/workflow.html').read_text().split('<main>',1)[1].split('</main>',1)[0]
        svg=re.search(r'<svg[\s\S]*?</svg>',content).group(0).replace('viewbox=','viewBox=').replace('font-size:initial;','font-size:initial;font-family:Arial,Helvetica,sans-serif;')
        (folder/f'{stem}.svg').write_text(svg)
        js="const sharp=require(process.argv[1]); sharp(process.argv[2],{density:220}).flatten({background:'#ffffff'}).png().toFile(process.argv[3]);"
        subprocess.run([str(RUNTIME/'node/bin/node'),'-e',js,str(RUNTIME/'node/node_modules/sharp'),str(folder/f'{stem}.svg'),str(folder/f'{stem}.png')],check=True)
        (folder/f'{stem}.html').write_text('<!doctype html><html lang="en"><meta charset="utf-8"><title>SharePoint workflow</title><style>body{font-family:Arial;color:#163c47;max-width:1100px;margin:30px auto}svg{width:100%;height:auto}@media print{@page{size:landscape}}</style><h1>Sponsor Highlights // SharePoint workflow</h1>'+svg+'<p>New originals and corrections retain separate immutable receipts and trigger best-effort Teams after verified export. Review and apply keep the latest accepted content. Existing reports remain unchanged. Dashed links indicate owner-triggered milestones; local CLI commands send no messages.</p><p>ACORN / ORNL pilot // October 27, 2026</p></html>')


def build(filename, pages, audience):
    target=ASSETS/filename
    c=Canvas(str(target),pagesize=(W,H))
    c.setTitle('Sponsor Highlights - '+audience);c.setAuthor('ACORN / ORNL pilot')
    for i,(title,section,draw) in enumerate(pages,1):
        header(c,title,section,i,len(pages),audience)
        c.bookmarkPage(f'p{i}')
        appendix=section.startswith('Appendix A')
        if appendix and (i==1 or not pages[i-2][1].startswith('Appendix A')):
            c.bookmarkPage('appendix-a')
            c.addOutlineEntry('Appendix A - GitLab connector','appendix-a',0)
        c.addOutlineEntry(title,f'p{i}',1 if appendix else 0)
        draw(c);c.showPage()
    c.save()
    return target



def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--user-only',action='store_true',help='Rebuild only the user/reviewer PDF and its Teams preview')
    args=parser.parse_args()
    render_diagram()
    render_diagram('correction-history')
    common=[('Submit and review with SharePoint','Start here',start),('From highlight to reviewed report','Workflow map',diagram)]
    user_pages=[common[0],('Allow the expected connections','First load',first_load),common[1]]+[
        ('Load the correct project list','Contributor // 01',projects),('Complete the form fields','Contributor // 02',fields),
        ('Choose the date and category','Form controls',field_menus),('Choose status and optional event type','Form controls',status_menus),
        ('Write, review and submit once','Contributor // 03',write_highlight),('Read the SharePoint confirmation','SharePoint receipt',sp_receipt),
        ('Recover without creating a duplicate','SharePoint recovery',recovery),
        ('Distinguish an uncertain result from a conflict','SharePoint recovery evidence',recovery_panels),('Use the form on a small screen','Responsive layout',mobile),
        ('Recognize the saved-highlight notice','Illustrative Teams messages',teams_mock),('Know when Teams and email are used','Notifications',notifications),
        ('Review before applying or publishing','Reviewer',reviewer),('Access, help and pilot boundaries','Reference',support),
        ('Correct a confirmed highlight','Contributor // Editing',editing),
        ('From correction history to current content','Correction workflow',correction_history),
        ('The retained GitLab connector route','Appendix A // Paused connector',branch_overview),
        ('Submit through GitLab after restoration','Appendix A // Historical form',gl_submit),
        ('Read a retained GitLab confirmation','Appendix A // Receipt evidence',gl_receipt),
        ('Handle an existing GitLab receipt','Appendix A // Recovery and notifications',gl_recovery)]
    dev_pages=common+[
        ('Locate the deployed SharePoint components','Developer // Inventory',components),('Understand the SharePoint project catalog','Developer // Catalog',catalogs),
        ('Preserve the SharePoint storage contract','SharePoint storage',sp_contract),('Read SharePoint run evidence','SharePoint acceptance',sp_run),
        ('Bring downloaded JSONL into local review','Local workflow',local_handoff),('Use the current graduation commands','Local workflow',graduation),
        ('Operate notifications independently of storage','Teams and email',delivery),('Separate proven behavior from release gates','Operations and backup',operations),
        ('Find the live app and retained evidence','Reference',references),
        ('Preserve correction history during intake','Correction workflow',correction_history),
        ('Locate the retained GitLab components','Appendix A // Paused connector',gl_inventory),
        ('Verify the GitLab commit and replay','Appendix A // Persistence evidence',gl_persistence),
        ('Restore and operate the GitLab route','Appendix A // Restoration',gl_restore)]
    user=build('sponsor-highlights-user-reviewer-guide.pdf',user_pages,'User and reviewer guide')
    dev=None if args.user_only else build('sponsor-highlights-developer-guide.pdf',dev_pages,'Developer and operations guide')
    with tempfile.TemporaryDirectory(prefix='pa-fonts-') as name:
        font_config=Path(name)/'fonts.conf'
        font_config.write_text('<fontconfig><dir>/System/Library/Fonts</dir><dir>/Library/Fonts</dir>'
                               f'<cachedir>{name}/cache</cachedir></fontconfig>')
        teams_page=next(i for i,(_,_,draw) in enumerate(user_pages,1) if draw is teams_mock)
        subprocess.run([str(RUNTIME/'bin/override/pdftoppm'),'-f',str(teams_page),'-singlefile','-scale-to','1600','-png',
                        str(user),str(ASSETS/'notifications/teams-example')],check=True,
                       env={**os.environ,'FONTCONFIG_FILE':str(font_config)})
    print(user)
    if dev: print(dev)


if __name__=='__main__':main()
