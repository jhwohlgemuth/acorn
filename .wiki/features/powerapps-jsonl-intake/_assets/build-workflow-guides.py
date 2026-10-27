#!/usr/bin/env python3
"""Build the two current guides and their notification extracts from retained evidence."""
import html
import os
import re
import shutil
import subprocess
import tempfile
from pathlib import Path

from pypdf import PdfReader, PdfWriter
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
    safe = '<br/>'.join(html.escape(line).replace(' ','&#160;') for line in text.splitlines())
    return paragraph(c, f'<font name="Courier">{safe}</font>', 68, top, 944, size)


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
    c.drawString(54,29,f'ACORN / ORNL PILOT  //  {audience}  //  OCT 26, 2026')
    c.drawRightString(1026,29,f'{number:02} / {total:02}')


def link(label, url):
    return f'<a href="{html.escape(url,quote=True)}" color="{GREEN}"><b>{label}</b></a>'


def diagram(c):
    paragraph(c,'Choose one intake route per highlight. Both preserve the original submission for later review.',54,153,size=17)
    picture(c,ASSETS/'notifications/workflow.png',85,192,910,465)
    paragraph(c,'Solid green: data handoff. Teams notices at save are automatic and best effort. Dashed: owner-triggered milestones; the local CLI sends no messages. A local PDF does not itself publish a report or send email.',54,676,size=14)


def branch_overview(c):
    paragraph(c,'Use the app link supplied by the owner. These are separate apps, not a branch selector within one form.',54,157)
    top=grid(c,[['','SharePoint pilot','GitLab connector'],
      ['Open',link('Sponsor Highlights - SharePoint Pilot',SP_APP),link('Sponsor Highlights',GL_APP)],
      ['Storage','One shared list row per receipt. Combined logbook.jsonl refreshes every five minutes; individual receipt files remain available.','One appended line in the repository JSONL logbook; receipt identifies a commit.'],
      ['Projects','13 seeded SharePoint catalog rows. Updates are maintained explicitly.','Repository configuration: .acorn.jsonc folders, or index.json / index.yaml discovery.'],
      ['Current scope','Owner tests passed. Standard-license participant access is still awaiting validation.','Existing operational pilot. Custom connector requires appropriate premium use rights; intermittent GitLab denials remain.'],
      ['After capture','Download combined logbook.jsonl for local ACORN review. Allow the next five-minute export run.','Stage the GitLab file URL or an exact local snapshot for ACORN review.'] ],198,size=15)
    paragraph(c,'Do not send the same pending highlight through both apps. The original GitLab app remains enabled; the SharePoint app does not replace it yet.',54,top,size=16,bold=True)


def projects(c):
    paragraph(c,'Sign in with your approved ORNL account. Check Submitted by, then load the project catalog and select the correct project.',54,156)
    paragraph(c,'The catalog may load automatically. The button reads <b>Load Project List</b> before its first successful load and <b>Refresh Project List</b> afterward. Loading alone does not submit anything.',54,218,size=16)
    picture(c,ASSETS/'sharepoint/projects-list.png',54,290,972,350)
    paragraph(c,'October 26 capture: the shared SharePoint catalog contains 13 project IDs. It is a seeded snapshot, not an automatic GitLab synchronization. If a project is absent, ask the owner; do not choose an unrelated project.',54,658,size=14)


def fields(c):
    top=grid(c,[['Field','What to enter'],['Project / date','Select the research project and the date the highlight describes.'],
     ['Category','administration, collaboration, data, engagement, funding, infrastructure, method, other, personnel, publication, research or software.'],
     ['Status','completed, in-progress or upcoming.'],['Event type','Optional: conference, exhibition, meeting, other, presentation, tour, training or workshop.'],
     ['Summary / details','Summary: required, up to 1,200 characters. Details: optional, one per line; up to 20 details, each up to 1,000 characters.'],
     ['Report placement','Project highlight, or General update (report title page). This chooses later placement; it does not publish.']],156,widths=(215,757),size=16)
    paragraph(c,'Both branches use the same form fields and envelope. Include a concrete result and why it matters. Use non-sensitive content because pending submissions are stored in the browser.',54,top,size=17)


def write_highlight(c):
    paragraph(c,'Review the project, date, category, status and text. Select <b>Submit highlight</b> once. The app freezes the payload and receipt ID while it waits for confirmation.',54,156)
    picture(c,ASSETS/'notifications/form-before-submit.png',54,220,972,445)
    paragraph(c,'Dated GitLab capture, October 24: a synthetic LandScan example. SharePoint uses the same entry fields with a SharePoint pilot subtitle and different save confirmation.',54,686,size=14)


def field_menus(c):
    paragraph(c,'Set the date of the work, then choose the category that best describes it. These shared controls work the same way in both intake branches.',54,156)
    picture(c,ASSETS/'walkthrough/07-date-picker.png',54,231,474,390)
    picture(c,ASSETS/'walkthrough/03-categories.png',552,231,474,390)
    paragraph(c,'Dated form captures, October 24. Left: calendar picker; confirm with Ok or use Cancel to retain the prior date. Right: category menu; scroll for additional choices. Earlier project labels in the captures are not the current catalog inventory.',54,661,size=15)


def status_menus(c):
    paragraph(c,'Status is required. Event type can remain Not specified if the highlight has no associated event. The same choices are used in both apps.',54,156)
    picture(c,ASSETS/'walkthrough/05-status-options.png',54,231,474,390)
    picture(c,ASSETS/'walkthrough/06-event-options.png',552,231,474,390)
    paragraph(c,'Dated form captures, October 24. Left: progress status. Right: optional event type. The form remains an intake surface; choosing Completed describes the work, not reviewer approval.',54,661,size=15)


def recovery_panels(c):
    paragraph(c,'An unresolved save and a conflicting receipt require different actions. Read the result before retrying or editing.',54,156)
    picture(c,ASSETS/'browser-recovery/jason-unresolved-panel.png',54,230,474,390)
    picture(c,ASSETS/'browser-recovery/jason-conflict-panel.png',552,230,474,390)
    paragraph(c,'Isolated PA-14 test app captures: unresolved outcome (left) and conflict (right). These simulated panels demonstrate the safety distinction; they are not proof of a new SharePoint retry or conflict. Preserve the same receipt for owner investigation.',54,661,size=15)


def mobile(c):
    paragraph(c,'On a small screen the form stacks vertically. Scroll through the same fields, review the selected project and use one tab while a submission is pending.',54,156)
    picture(c,ASSETS/'visual-polish-phone-top.png',54,225,474,410)
    picture(c,ASSETS/'visual-polish-phone-bottom.png',552,225,474,410)
    paragraph(c,'Dated responsive previews of the original form. They illustrate the shared layout, not native-mobile or SharePoint mobile-storage acceptance. The lower section contains report placement, review guidance and Submit highlight.',54,661,size=15)


def sp_receipt(c):
    paragraph(c,'<b>Highlight saved · JSONL ready</b> means the list receipt exists and the exported file passed exact readback. Keep the receipt ID. Saving is separate from content approval and publication.',54,156)
    picture(c,ASSETS/'sharepoint/app-teams-success.png',54,218,972,447)
    paragraph(c,'October 26 published-app test: receipt 64b836e0-016f-4493-9894-a6fe79acc8de. Receipt, file verification and Teams action succeeded. Recipient reading of the message was not independently checked.',54,681,size=14)


def gl_receipt(c):
    paragraph(c,'<b>Highlight saved</b> confirms the exact submission in the GitLab logbook at the displayed commit. Keep both the receipt ID and commit reference; a catalog load alone is not a saved highlight.',54,156)
    picture(c,ASSETS/'notifications/form-confirmed.png',54,220,972,447)
    paragraph(c,'Dated published acceptance, October 24: receipt 82a8dd58-8c60-4ec5-abf5-cb251ab9e033 at commit 03ff58362894. Later GitLab network denials remain a known limitation.',54,681,size=14)


def recovery(c):
    top=grid(c,[['Result','SharePoint pilot','GitLab connector'],
      ['Confirmed','Receipt plus verified export. Back to highlight preserves the locked entry.','Receipt plus immutable commit. Back to highlight preserves the locked entry.'],
      ['Received; export pending','Keep the same ID. Use Retry JSONL export; the list receipt alone is not an export confirmation.','Not used by this branch. Read its commit / unresolved result.'],
      ['Uncertain save','Keep the frozen content and retry the existing submission. Do not change apps.','Retry the same ID and content so prior immutable history can be reconciled.'],
      ['Rejected / conflict','Edit only after explicit rejection permits it. Conflicting ID/content needs owner review.','Same rule: rejection permits correction; conflict needs owner review.'],
      ['Browser recovery unavailable','Read the message; a new send is stopped if pending storage cannot be prepared.','Same rule. Do not clear existing browser data while a result is unresolved.']],156,size=14.5)
    paragraph(c,'Use one tab and the same browser profile/account. New highlight clears only that app\'s recovery slot; it does not delete the stored receipt. SharePoint reload passed; its full restart, denied-storage and pending-export recovery acceptance remain open.',54,top,size=15)


def teams_mock(c):
    paragraph(c,'Fictional chats and receipts below illustrate both routes. No personal Teams screenshot is used.',54,155,size=16)
    c.setFillColor(HexColor('#484772'));c.roundRect(54,145,972,450,10,fill=1,stroke=0)
    paragraph(c,'Teams  //  Pilot owner (you)  //  Example workflow updates',74,228,920,16,'#FFFFFF',True)
    for x,branch,status,target in [(74,'SHAREPOINT PILOT','Received and exported','View JSONL export'),(554,'GITLAB CONNECTOR','Saved to the logbook','View saved record')]:
        c.setFillColor(white);c.roundRect(x,H-292-335,452,335,8,fill=1,stroke=0)
        y=paragraph(c,branch,x+22,310,410,11,GREEN,True)
        y=paragraph(c,'ACORN // Sponsor Highlights',x+22,y,410,13)
        y=paragraph(c,'Highlight saved',x+22,y,410,24,INK,True)
        y=paragraph(c,'<b>Project</b><br/>Example Project',x+22,y,410,15)
        y=paragraph(c,'<b>Status</b><br/>'+status+'<br/>Awaiting ACORN review',x+22,y,410,15)
        y=paragraph(c,target,x+22,y,410,16,GREEN,True)
        paragraph(c,'Receipt ID: EXAMPLE-001',x+22,y,410,12)
    paragraph(c,'Link destination differs: the SharePoint JSONL file or the immutable GitLab commit. Real notices go to the pilot owner, not every contributor. A missing notice does not undo a confirmed save.',54,681,size=14)


def notifications(c):
    top=grid(c,[['Milestone','Channel','When it happens today'],
      ['SharePoint new receipt + export','Teams to owner','Automatic after response handling, only when this run created the receipt and verified its export.'],
      ['GitLab confirmed new save','Teams to owner','Automatic after confirmed new-save response. Exact replay does not resend.'],
      ['Workspace created / no report changes','Teams to owner','Owner invokes the manual callback for an actual milestone.'],
      ['Review requested / report ready / actionable failure','Email to owner','Owner invokes the manual callback with the real review or deliverable evidence.'],
      ['Typing, validation or unconfirmed save','App feedback only','No routine intake email; no new notice for exact replays.']],156,size=15)
    paragraph(c,'SharePoint notification limitation: an export that first succeeds on a later retry does not send the initial new-receipt notice. Delivery is best effort. Local CLI commands do not invoke these callbacks; Vergil automation remains separate work.',54,top,size=16)


def reviewer(c):
    top=paragraph(c,'The form collects submissions; it does not provide an approval screen. The owner supplies the proposed changes and review location.',54,156)
    for title,body in [('1 // Check the evidence','Match the project and receipt. Review date, category, status, placement, summary and details against the underlying work.'),('2 // Inspect the local proposal','Both routes feed the same ACORN stage / inspect / apply process. Confirm the proposed diff before allowing the destination index to change.'),('3 // Confirm optional promotion','Graduation may add a contributor or other supported metadata. Use the interactive run --index mode for per-entry Yes / No selection and a separate final confirmation.'),('4 // Review the report','A local PDF is a preview, not publication. Follow the supplied review process before merging changes or releasing a report.')]:
        top=paragraph(c,title,54,top,size=20,bold=True)
        top=paragraph(c,body,54,top,size=17)
    paragraph(c,'Ask the owner to coordinate corrections to confirmed records. Do not edit stored receipt payloads or create duplicate submissions just to retry reporting.',54,top,size=16,color=GREEN,bold=True)


def support(c):
    top=paragraph(c,'If access fails, keep the exact error and app link. A standard connector does not grant an account entitlement, app sharing, site access or environment permission.',54,156)
    top=grid(c,[['Check','Current position'],['SharePoint participant access','Owner-context tests passed. Another intended standard-license user still needs an end-to-end access test before wider sharing.'],['GitLab access','Custom connector use requires appropriate premium use rights. Intermittent network-range denials are recorded as a known limitation.'],['Help request','Send the owner the app/branch, receipt ID, selected project, approximate time and visible result. Never send a token or API key.'],['Pending submission','Keep its original app, ID and content. No automatic migration or duplicate submission across branches.']],top,widths=(240,732),size=16)
    paragraph(c,link('Open SharePoint pilot',SP_APP)+'  |  '+link('Open GitLab app',GL_APP)+'  |  '+link('ACORN intake reference',DOCS),54,top,size=16)
    paragraph(c,'Microsoft references: '+link('SharePoint connector', 'https://learn.microsoft.com/en-us/connectors/sharepointonline/')+'; '+link('Power Apps license designation','https://learn.microsoft.com/en-us/power-apps/maker/canvas-apps/license-designation')+'. Pilot acceptance is recorded in sharepoint-pilot.md and the active PA plan.',54,top+57,size=13)


def components(c):
    rows=[['Component','SharePoint pilot','GitLab connector'],['Canvas app','163f110f-7f8a-401c-894e-0786b2c018a4','15ae605b-585b-49f5-a413-2b091f919a35'],['Intake flow','64679e9c-ae76-4263-b8fa-8832835516ca','ff4a134b-5fd3-4465-ab19-0325c2f2bf1b'],['Catalog flow','6f8a4ab5-dd66-4904-9f5d-f307519ab4d5','cd27888f-dd74-4675-9dd1-abc246ba56d5'],['Browser slot','SponsorHighlightSharePointPending','SponsorHighlightPending'],['Destination','Research Enablement site: Intake Pilot list and JSONL Pilot library','Project 22464 / main / .acorn/intake/logbook.jsonl']]
    top=grid(c,rows,156,widths=(180,396,396),size=14)
    top=paragraph(c,'Environment: <b>Wohlgemuth, Jason</b> ('+ENV+'). The SharePoint app binds only its two pilot flows. Both original GitLab flows remain enabled and unchanged.',54,top,size=16)
    paragraph(c,'Owner-only callback flow (separate from both app intake bindings): dd1d8f67-116c-f55d-7752-7c7d322691c5. It retains Dataverse receipt tracking and is not the standard-only contributor route.',54,top,size=15)


def sp_contract(c):
    top=grid(c,[['Field / behavior','Contract'],['SubmissionId','Required, unique 36-character UUID. Find/read the receipt before confirming; changed payload under the same UUID conflicts.'],['PayloadBase64A / B','Base64 of exact original UTF-8 JSON. A holds the first 45,000 encoded characters; B holds the remainder. Encoding is not encryption.'],['Display fields','ProjectId, Summary and SubmitterEmail are for viewing; do not reconstruct the export from them.'],['JSONL export','Receipt: decode A + B, add LF and verify UUID.jsonl. Combined: one serialized five-minute flow rebuilds logbook.jsonl from list rows, verifies exact bytes and skips unchanged writes.'],['ExportStatus / URL','Pending until exact readback succeeds; then Exported with the confirmed file location. A receipt alone is not a successful export.']],156,widths=(230,742),size=15)
    paragraph(c,'Server checks include envelope shape, raw size/newlines, caller identity, timestamps and exact catalog membership. Combined export caps at 5,000 rows and 10 MiB, retaining the prior file on validation failure. Boundary and concurrent-arrival acceptance remains open. Standard SharePoint HttpRequest is used; no premium generic HTTP action is introduced.',54,top,size=15)


def sp_run(c):
    top=paragraph(c,'Read action outcomes, not just the overall run color. A replay can fail non-overwrite file creation while successfully reading and confirming the existing exact file.',54,156)
    top=grid(c,[['Case','Observed October 26','Run suffix'],['Fresh app + Teams','Receipt, export/readback, response and Teams action succeeded.','...94369117892624466851CU16'],['Exact replay after Teams','Receipt creation skipped; existing file matched; Teams skipped.','...92740963250031363046CU21'],['Changed payload','Conflict branch succeeded; receipt/file writes skipped.','...11980621045990695644CU06'],['Malformed envelope','Parse failed; persistence skipped; outer response succeeded.','...11189995757802689639CU25']],top,widths=(185,535,252),size=14)
    paragraph(c,'Response outputs: status, submissionid, exportstatus, exportfileurl and receipturl. The app accepts exported only with a matching UUID, received status, exported state and expected file URL. Uncertain export retains the frozen payload and offers Retry JSONL export.',54,top,size=16)
    paragraph(c,'Full run IDs and action evidence: sharepoint-pilot.md and _assets/sharepoint/. Secure history hides response values; the published app confirmed the successful response contract. Failure/retry panels still require app-driven acceptance.',54,top+95,size=14)


def catalogs(c):
    top=grid(c,[['','SharePoint','GitLab'],['Source','13 rows seeded from retained revision 03ff5836289447034f40f2a4898c155441e6cb66. No automatic sync.','.acorn.jsonc folders is authoritative, including an empty list. Without it, find folders with index.json or index.yaml.'],['Response','catalogjson text: source=sharepoint, projects=[{id,label}]. More than 1,000 rows returns catalog_limit with an empty list.','Choices correspond to canonical folders at one repository revision; invalid config or denied access stops discovery.'],['Acceptance','Owner manual and app loads passed. Overflow is authored but not tested.','Successful reads do not prove later writes; network restrictions may be intermittent.']],156,size=15)
    picture(c,ASSETS/'sharepoint/catalog-success.png',54,top,972,240)
    paragraph(c,'October 26 catalog flow capture. Catalog membership does not replace ACORN folder/member authorization during later admission.',54,696,size=13)


def gl_persistence(c):
    paragraph(c,'Validate the caller/envelope, resolve the project, check immutable history for an exact replay, then conditionally append one line. Preserve every previous byte and verify the committed snapshot before success.',54,156)
    picture(c,ASSETS/'published-append-commit.png',54,225,972,385)
    paragraph(c,'Dated append evidence. Exact replays return the verified prior commit without another append or Teams notice. History lookup is bounded; missing proof remains unresolved. Preserve the ID and payload across timeouts or network denials.',54,630,size=16)
    paragraph(c,'PA-02 is closed under the owner-approved known-limitation decision. That decision does not establish reliable writes or close the remaining boundary, concurrency or restore tests.',54,697,size=13)


def local_handoff(c):
    top=paragraph(c,'SharePoint: download the combined logbook.jsonl after the next five-minute refresh. Preserve the selected original rows and UUIDs. The owner-context three-row local report passed October 26; intended-user and broader export acceptance remain open.',54,156,size=15)
    top=paragraph(c,'From the prepared demo directory, use acorn on PATH. intake.json and report.json point to the same separate demo index. Replace BATCH_ID below with the value printed by stage.',54,top,size=15)
    top=code(c,'cd output/pdf/sharepoint-demo-20261026\n'
        '# 1. Stage the three exact selected rows.\n'
        'acorn --offline intake stage --batch --config intake.json \\\n  --source selected-demo.jsonl\n'
        '# 2. Inspect; 3. apply with the normal Yes / No buttons.\n'
        'acorn --offline intake inspect --config intake.json --batch-id BATCH_ID\n'
        'acorn --offline intake apply --config intake.json --batch-id BATCH_ID\n'
        '# 4. Generate the combined local PDF.\n'
        'acorn --offline report --config report.json \\\n  --from 2026-10-26T00:00:00Z --through 2026-10-27T00:00:00Z \\\n  --dry-run --combine',top,13)
    top=paragraph(c,'Open .acorn/reports/report.pdf. Defaults use the current directory and all configured projects (only acorn here). Combined mode needs no commit. Keep the fixed date window; --through has no time default. Set CHROME_PATH only if browser discovery fails.',54,top,size=14)
    paragraph(c,'GitLab file URLs use online stage with CI_SERVER_HOST and the existing protected credential reference, then the same local inspect / apply / report steps. Direct SharePoint URLs remain planned. Local reporting sends no messages or callbacks; optional graduation is separate.',54,top,size=14)


def graduation(c):
    top=paragraph(c,'After ingestion, use the current local graduation API. Supported entries can propose contributor or other metadata changes; unsupported entries remain unchanged.',54,156)
    top=code(c,'acorn intake run --index project/index.json --dry-run\nacorn intake run --index project/index.json --interactive\nacorn intake run --index project/index.json --force',top,16)
    top=grid(c,[['Mode','Meaning'],['--dry-run','Validate and preview; write nothing.'],['--interactive','Per-candidate Yes / No selection, then separate final write confirmation. No is the default.'],['--force','Accept all eligible promotions without prompts; validation, digest checks and exact backup remain.'],['--accept ID --expected-digest DIGEST','Apply only an explicitly reviewed selection with the pinned digest.']],top,widths=(340,632),size=15)
    paragraph(c,'Promotion retains and archives source entries and writes an adjacent exact-byte backup. A bare run --index with eligible entries requires an explicit approval mode. Local run --index is distinct from the GitLab worker run --config. Neither local ingestion nor graduation sends milestone messages.',54,top,size=15)


def delivery(c):
    top=grid(c,[['Route','Automatic Teams gate','Recovery limit'],['SharePoint','Create_receipt_SP and Mark_exported_SP must both succeed in the same run. Runs after response handling.','Exact replay skips. Export first confirmed on a later retry also skips the initial notice; no durable notice outbox.'],['GitLab','Queued_new must succeed. Runs after response handling.','Replay, rejection and unresolved results skip. A later notice failure does not roll back the saved commit.']],156,size=15)
    top=paragraph(c,'Both use the existing Teams self-notification connection, secure action inputs/outputs and no automatic send retries. Notices omit summary/details and link to the stored record. Do not resubmit confirmed intake merely to generate a message.',54,top,size=16)
    top=grid(c,[['Callback event','Required additional fields','Channel'],['mr-created / review-needed','merge_request_url, project_id','Teams / email'],['report-no-changes','project_id, through','Teams'],['artifact-ready','checksum, package_url, project_id, through','Email'],['failed','message, project_id','Email']],top,widths=(275,520,177),size=14)
    paragraph(c,'Manual callback: exact Webhook ID + Event JSON. Unique claim; frozen payload and destination; Pending > Sending > Sent; concurrency one. Sent replay skips. Uncertain Sending holds for reconciliation against the recorded attempt. Never reset solely because a send timed out.',54,top,size=14)


def operations(c):
    top=grid(c,[['Area','Current acceptance and next action'],['SharePoint pilot','Owner app submit, catalog, exact export, reload and new Teams action passed. Manual replay/conflict/malformed handling passed.'],['Remaining SharePoint gates','PA-19 to PA-21: catalog overflow, max sizes, concurrency, export-failure recovery, app conflict/replay, full restart and denied storage. PA-22: intended-participant access/handoff; the owner three-row report passed. PA-24/25: backup/restore and optional cutover. PA-23: combined-export boundary and arrival tests.'],['GitLab gates','PA-15: remaining tail/size cases; PA-16: concurrent writes; PA-17: complete restore acceptance. PA-02 closure records intermittent connectivity as a known limitation.'],['Backup','Native GitLab packages exist; full restore acceptance remains open. SharePoint solution contains its flows/reference; app inclusion and complete native export/restore are pending. Clipboard JSON/YAML is not a native backup.'],['Release boundary','Pilot remains in Wohlgemuth, Jason. Site membership unchanged. Owner premium access does not prove standard-user access. PA-12 shared-environment/channel migration remains separate.']],156,widths=(245,727),size=15)
    paragraph(c,'Keep original pending receipts in their original app. Do not disable the GitLab route or widen sharing as part of documentation work. Teams/email are notifications; stored receipt, exact source and reviewed index are the evidence.',54,top,size=15)


def references(c):
    top=paragraph(c,'Working links',54,156,size=21,bold=True)
    for label,url in [('SharePoint pilot app',SP_APP),('Original GitLab app',GL_APP),('Research Enablement SharePoint site',SP_SITE),('Pilot GitLab repository',REPO),('ACORN intake command reference',DOCS),('Microsoft SharePoint connector classification','https://learn.microsoft.com/en-us/connectors/sharepointonline/'),('Microsoft Power Apps license designation and linked-flow caveat','https://learn.microsoft.com/en-us/power-apps/maker/canvas-apps/license-designation')]:
        top=paragraph(c,link(label,url),54,top,size=16)
    top=paragraph(c,'Repository evidence',54,top+5,size=21,bold=True)
    paragraph(c,'Within .wiki/features/powerapps-jsonl-intake/: sharepoint-pilot.md, notification-walkthrough.md, boundary-acceptance.md, restore-target-readiness.md and index.md. Manual workflow: ../manual-jsonl-ingestion-graduation/walkthrough.md and handback.md.<br/><br/>Screenshots are dated evidence, not a promise of current availability. Synthetic records remain TEST ONLY. The full PDFs were regenerated October 26 with both branches and current CLI commands.',54,top,size=16)


def render_diagram():
    folder = ASSETS/'notifications'
    with tempfile.TemporaryDirectory(prefix='pa-guides-') as name:
        root=Path(name);(root/'src').mkdir()
        (root/'book.toml').write_text('[book]\ntitle="Workflow"\n[preprocessor.pikchr]\ncommand="'+str(Path.home()/'.cargo/bin/mdbook-pikchr')+'"\n')
        (root/'src/SUMMARY.md').write_text('# Summary\n- [Workflow](workflow.md)\n')
        (root/'src/workflow.md').write_text('```pikchr\n'+(folder/'workflow.pikchr').read_text()+'\n```')
        subprocess.run([shutil.which('mdbook') or str(Path.home()/'.cargo/bin/mdbook'),'build',str(root)],check=True)
        content=(root/'book/workflow.html').read_text().split('<main>',1)[1].split('</main>',1)[0]
        svg=re.search(r'<svg[\s\S]*?</svg>',content).group(0).replace('viewbox=','viewBox=').replace('font-size:initial;','font-size:initial;font-family:Arial,Helvetica,sans-serif;')
        (folder/'workflow.svg').write_text(svg)
        js="const sharp=require(process.argv[1]); sharp(process.argv[2],{density:220}).flatten({background:'#ffffff'}).png().toFile(process.argv[3]);"
        subprocess.run([str(RUNTIME/'node/bin/node'),'-e',js,str(RUNTIME/'node/node_modules/sharp'),str(folder/'workflow.svg'),str(folder/'workflow.png')],check=True)
        (folder/'workflow.html').write_text('<!doctype html><html lang="en"><meta charset="utf-8"><title>Two intake branches</title><style>body{font-family:Arial;color:#163c47;max-width:1100px;margin:30px auto}svg{width:100%;height:auto}@media print{@page{size:landscape}}</style><h1>Sponsor Highlights // Two intake branches</h1>'+svg+'<p>Choose one branch per highlight. Save notices are automatic and best effort. Dashed links indicate owner-triggered milestones; the local CLI sends no messages. A local PDF does not itself publish or send a report.</p><p>ACORN / ORNL pilot // October 26, 2026</p></html>')


def build(filename, pages, audience):
    target=ASSETS/filename
    c=Canvas(str(target),pagesize=(W,H))
    c.setTitle('Sponsor Highlights - '+audience);c.setAuthor('ACORN / ORNL pilot')
    for i,(title,section,draw) in enumerate(pages,1):
        header(c,title,section,i,len(pages),audience)
        c.bookmarkPage(f'p{i}');c.addOutlineEntry(title,f'p{i}',0)
        draw(c);c.showPage()
    c.save()
    return target


def extract(source, indexes, output):
    reader=PdfReader(source);writer=PdfWriter()
    for i in indexes:writer.add_page(reader.pages[i])
    writer.add_metadata({'/Title':'Sponsor Highlights - Workflow and notification extract','/Author':'ACORN / ORNL pilot'})
    with output.open('wb') as stream:writer.write(stream)


def main():
    render_diagram()
    common=[('One workflow, two intake branches','Start here',branch_overview),('From highlight to reviewed report','Workflow map',diagram)]
    user=build('sponsor-highlights-user-reviewer-guide.pdf',common+[
        ('Load the correct project list','Contributor // 01',projects),('Complete the shared form fields','Contributor // 02',fields),
        ('Choose the date and category','Shared controls',field_menus),('Choose status and optional event type','Shared controls',status_menus),
        ('Write, review and submit once','Contributor // 03',write_highlight),('Read the SharePoint confirmation','SharePoint branch',sp_receipt),
        ('Read the GitLab confirmation','GitLab branch',gl_receipt),('Recover without creating a duplicate','Both branches // Recovery',recovery),
        ('Distinguish an uncertain result from a conflict','Dated recovery evidence',recovery_panels),('Use the form on a small screen','Responsive layout',mobile),
        ('Recognize the saved-highlight notice','Illustrative Teams messages',teams_mock),('Know when Teams and email are used','Notifications',notifications),
        ('Review before applying or publishing','Reviewer',reviewer),('Access, help and pilot boundaries','Reference',support)],'User and reviewer guide')
    dev=build('sponsor-highlights-developer-guide.pdf',common+[
        ('Locate the deployed components','Developer // Inventory',components),('Understand each project catalog','Developer // Catalogs',catalogs),
        ('Preserve the SharePoint storage contract','SharePoint branch',sp_contract),('Read SharePoint run evidence','SharePoint branch // Acceptance',sp_run),
        ('Verify the GitLab commit and replay','GitLab branch',gl_persistence),('Bring either JSONL source into local review','Shared local workflow',local_handoff),
        ('Use the current graduation commands','Shared local workflow',graduation),('Operate notifications independently of storage','Teams and email',delivery),
        ('Separate proven behavior from release gates','Operations and backup',operations),('Find the live apps and retained evidence','Reference',references)],'Developer and operations guide')
    extract(user,[0,1,12,13],ASSETS/'notifications/user-reviewer-notification-update.pdf')
    extract(dev,[0,1,9,10],ASSETS/'notifications/developer-notification-update.pdf')
    with tempfile.TemporaryDirectory(prefix='pa-fonts-') as name:
        font_config=Path(name)/'fonts.conf'
        font_config.write_text('<fontconfig><dir>/System/Library/Fonts</dir><dir>/Library/Fonts</dir>'
                               f'<cachedir>{name}/cache</cachedir></fontconfig>')
        subprocess.run([str(RUNTIME/'bin/override/pdftoppm'),'-f','13','-singlefile','-scale-to','1600','-png',
                        str(user),str(ASSETS/'notifications/teams-example')],check=True,
                       env={**os.environ,'FONTCONFIG_FILE':str(font_config)})
    print(user);print(dev)


if __name__=='__main__':main()
