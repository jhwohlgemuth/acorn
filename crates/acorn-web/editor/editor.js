import { defaultKeymap, history, historyKeymap } from "@codemirror/commands";
import { javascript } from "@codemirror/lang-javascript";
import { json } from "@codemirror/lang-json";
import { markdown } from "@codemirror/lang-markdown";
import { xml } from "@codemirror/lang-xml";
import { yaml } from "@codemirror/lang-yaml";
import { bracketMatching, foldable, foldAll, foldEffect, foldGutter, foldKeymap, forceParsing, indentOnInput, StreamLanguage, syntaxHighlighting, unfoldAll } from "@codemirror/language";
import { toml } from "@codemirror/legacy-modes/mode/toml";
import { turtle } from "@codemirror/legacy-modes/mode/turtle";
import { Compartment, EditorState, RangeSet, StateEffect, StateField } from "@codemirror/state";
import { openSearchPanel, search, searchKeymap } from "@codemirror/search";
import { Decoration, drawSelection, EditorView, GutterMarker, highlightActiveLine, highlightActiveLineGutter, hoverTooltip, keymap, lineNumberMarkers, lineNumbers } from "@codemirror/view";
import { classHighlighter } from "@lezer/highlight";
import { createEditorPopouts } from "./popout.js";

const languages = {
    expression: StreamLanguage.define({ token(stream) {
        if (stream.eatSpace()) return null;
        if (stream.match(/^(?:log|ln|exp|sqrt|O)\b/)) return "keyword";
        if (stream.match(/^\d+(?:\.\d+)?/)) return "number";
        if (stream.match(/^[a-zA-Z]+/)) return "variableName";
        stream.next(); return "operator";
    } }),
    identifier: StreamLanguage.define({ token(stream) {
        if (stream.match(/^(?:https?|doi|orcid|ror|ark|urn)(?=:)/i)) return "keyword";
        if (stream.match(/^\d+/)) return "number";
        stream.next(); return null;
    } }),
    javascript: javascript(), json: json(), markdown: markdown(),
    toml: StreamLanguage.define(toml), turtle: StreamLanguage.define(turtle), xml: xml(), yaml: yaml(),
};
function languageName(language, text) {
    const aliases = { cff: "yaml", "json-ld": "json", jsonc: "javascript", markup: "xml", "rdf-xml": "xml", text: "markdown", zon: "javascript" };
    const selected = aliases[language] || language;
    return selected === "auto" ? (/^\s*[[{]/.test(text) ? "json" : /^\s*</.test(text) ? "xml" : "markdown") : selected;
}
function foldDocument(view) {
    // Include offscreen regions after loading a file, while bounding synchronous parser work.
    forceParsing(view, view.state.doc.length, 100);
    const { state } = view;
    const effects = [];
    // Walk document lines, including those hidden inside existing or newly collected folds.
    for (let number = 1; number <= state.doc.lines; number++) {
        const line = state.doc.line(number);
        const range = foldable(state, line.from, line.to);
        if (range) effects.push(foldEffect.of(range));
    }
    if (effects.length) view.dispatch({ effects });
    return effects.length > 0;
}
const setDiagnosticMarks = StateEffect.define();
const pidDocumentation = {
    ark: "https://arks.org/about/ark-overview/",
    arxiv: "https://info.arxiv.org/help/arxiv_identifier.html",
    doi: "https://www.doi.org/the-identifier/resources/handbook/",
    handle: "https://handle.net/documentation.html",
    isbn: "https://www.isbn-international.org/content/what-isbn/10",
    isni: "https://www.iso.org/standard/87177.html",
    orcid: "https://info.orcid.org/what-is-orcid/",
    patent: "https://www.uspto.gov/patents/apply/applying-online/patent-number",
    raid: "https://documentation.raid.org/raid/what-is-raid",
    ror: "https://ror.org/about/faqs/",
    rrid: "https://www.rrids.org/rrid-system",
    swhid: "https://www.swhid.org/specification/v1.2/",
};
const emptyDiagnostics = { marks: Decoration.none, lines: RangeSet.empty, issues: [], discoveries: [] };
class DiagnosticLineMarker extends GutterMarker {
    constructor(severity) { super(); this.elementClass = `cm-diagnostic-line cm-diagnostic-line-${severity}`; }
}
const diagnosticLineMarkers = { error: new DiagnosticLineMarker("error"), warning: new DiagnosticLineMarker("warning") };
const diagnosticMarks = StateField.define({
    create: () => emptyDiagnostics,
    update(diagnostics, transaction) {
        if (transaction.docChanged) return emptyDiagnostics;
        return transaction.effects.find(effect => effect.is(setDiagnosticMarks))?.value ?? diagnostics;
    },
    provide: field => [EditorView.decorations.from(field, value => value.marks), lineNumberMarkers.from(field, value => value.lines)],
});
function diagnosticTooltip(issue) {
    const discovered = issue.severity === "discovered";
    const dom = document.createElement("div");
    dom.className = "cm-diagnostic-tooltip";
    dom.dataset.severity = issue.severity;
    if (discovered) dom.dataset.pid = issue.kind;
    dom.setAttribute("role", "tooltip");
    const heading = document.createElement("div");
    heading.className = "cm-diagnostic-tooltip-heading";
    const badge = document.createElement("strong");
    badge.className = "cm-diagnostic-tooltip-badge";
    badge.textContent = discovered ? issue.kind.toUpperCase() : issue.severity === "error" ? "Error" : "Warning";
    const location = document.createElement("span");
    location.textContent = `Line ${issue.line}`;
    heading.append(badge, location);
    dom.append(heading);
    if (issue.path || discovered) {
        const path = document.createElement("div");
        path.className = "cm-diagnostic-tooltip-path";
        path.textContent = discovered ? issue.normalized : issue.path;
        dom.append(path);
    }
    for (const message of issue.messages || []) {
        const body = document.createElement("p");
        body.textContent = message;
        dom.append(body);
    }
    if (discovered && pidDocumentation[issue.kind]) {
        const link = document.createElement("a");
        link.href = pidDocumentation[issue.kind];
        link.target = "_blank";
        link.rel = "noopener noreferrer";
        link.textContent = `About ${issue.kind.toUpperCase()} ↗`;
        dom.append(link);
    }
    return { dom };
}
const diagnosticHover = hoverTooltip((view, pos) => {
    const diagnostics = view.state.field(diagnosticMarks);
    const issue = diagnostics.issues.find(({ from, to }) => from <= pos && pos <= to)
        ?? diagnostics.discoveries.find(({ from, to }) => from <= pos && pos <= to);
    return issue ? { pos: issue.from, end: issue.to, above: true, create: () => diagnosticTooltip(issue) } : null;
}, { hideOnChange: true, hideOn: transaction => transaction.effects.some(effect => effect.is(setDiagnosticMarks)) });
function sourcePosition(doc, lineNumber, characterNumber) {
    if (!Number.isInteger(lineNumber) || lineNumber < 1 || lineNumber > doc.lines || !Number.isInteger(characterNumber) || characterNumber < 1) return null;
    const line = doc.line(lineNumber);
    const characters = Array.from(line.text);
    return characterNumber <= characters.length + 1 ? line.from + characters.slice(0, characterNumber - 1).join("").length : null;
}
function installEditors() {
    const editors = new Map();
    const popouts = createEditorPopouts();
    function setFullscreen(host, active) {
        const id = host.querySelector("textarea")?.id;
        const locked = popouts.isPopout(id);
        if (locked && !active) return;
        host.classList.toggle("is-fullscreen", active);
        document.body.classList.toggle("editor-fullscreen-open", active);
        const button = host.querySelector(".editor-fullscreen-toggle");
        if (button) {
            button.disabled = locked;
            button.setAttribute("title", locked ? "Full screen is fixed in this tab" : active ? "Exit full screen (Esc)" : "Full screen (Esc to exit)");
            button.setAttribute("aria-label", active ? "Exit full screen" : "Full screen");
        }
        const editor = find(id);
        if (editor) updateIssueControls(host, editor);
        editor?.view.requestMeasure();
    }
    function updateIssueControls(host, editor) {
        const controls = host.querySelector(".editor-issue-nav");
        if (!controls) return;
        const count = editor.issueLocations.length;
        const displayedIndex = count && editor.issueIndex < 0 && host.classList.contains("is-fullscreen") ? 0 : editor.issueIndex;
        controls.hidden = count === 0;
        const position = controls.querySelector(".editor-issue-position");
        const label = `${displayedIndex + 1} of ${count}`;
        if (position && position.textContent !== label) position.textContent = label;
        const message = controls.querySelector(".editor-issue-message");
        const active = editor.issueLocations[displayedIndex];
        const description = active ? `${active.severity === "error" ? "Error" : "Warning"}, line ${active.line}: ${active.messages.join("; ")}` : "";
        if (message && message.textContent !== description) message.textContent = description;
        if (message) message.dataset.severity = active?.severity || "";
        if (description) controls.title = description;
        else controls.removeAttribute("title");
    }
    function mount(host) {
        if (editors.has(host)) return;
        const input = host.querySelector("textarea");
        const surface = host.querySelector(".editor-surface");
        if (!input || !surface) return;
        const language = new Compartment();
        const wrapping = new Compartment();
        let accessibleLabel = input.getAttribute("aria-label") || "Code";
        let mode = languageName(host.dataset.language, input.value);
        let external = false;
        let editVersion = 0;
        let wrapped = true;
        const extensions = [
            EditorState.readOnly.of(input.readOnly), EditorView.editable.of(!input.readOnly),
            lineNumbers(), foldGutter(), history(), drawSelection(), diagnosticMarks, diagnosticHover, bracketMatching(), indentOnInput(), search(),
            highlightActiveLine(), highlightActiveLineGutter(),
            syntaxHighlighting(classHighlighter),
            keymap.of([...searchKeymap, ...foldKeymap.map(binding => binding.run === foldAll ? { ...binding, run: foldDocument } : binding), ...defaultKeymap, ...historyKeymap]),
            EditorView.contentAttributes.of(() => ({ id: `${input.id}-editor`, "aria-label": accessibleLabel, "aria-readonly": String(input.readOnly), tabindex: "0", spellcheck: "false", autocapitalize: "off" })),
            EditorView.updateListener.of((update) => {
                if (!update.docChanged || external) return;
                const editor = editors.get(host);
                if (editor) {
                    editor.issueLocations = [];
                    editor.issueIndex = -1;
                    updateIssueControls(host, editor);
                }
                const text = update.state.doc.toString();
                const version = ++editVersion;
                input.value = text;
                queueMicrotask(() => {
                    if (version !== editVersion || !host.isConnected) return;
                    input.value = text;
                    input.dispatchEvent(new Event("input", { bubbles: true }));
                });
            }),
        ];
        const state = (text) => EditorState.create({ doc: text, extensions: [extensions, language.of(languages[mode] || []), wrapping.of(wrapped ? EditorView.lineWrapping : [])] });
        const view = new EditorView({ parent: surface, state: state(input.value) });
        const grip = document.createElement("div");
        grip.className = "editor-resize-handle";
        grip.tabIndex = 0;
        grip.setAttribute("role", "separator");
        grip.setAttribute("aria-orientation", "horizontal");
        grip.setAttribute("aria-label", `Resize ${accessibleLabel} editor`);
        const minimumHeight = matchMedia("(pointer: coarse)").matches ? 120 : 86;
        grip.setAttribute("aria-valuemin", String(minimumHeight));
        grip.setAttribute("aria-valuemax", "2048");
        grip.title = "Drag to resize · Up/Down arrows to adjust height";
        function resize(height) {
            const size = Math.round(Math.max(minimumHeight, Math.min(2048, height)));
            surface.style.height = `${size}px`;
            grip.setAttribute("aria-valuenow", String(size));
            grip.setAttribute("aria-valuetext", `${size} pixels`);
            view.requestMeasure();
        }
        let drag;
        grip.addEventListener("pointerdown", event => {
            if (event.button !== 0 || !event.isPrimary) return;
            event.preventDefault();
            grip.focus({ preventScroll: true });
            drag = { y: event.clientY, height: surface.getBoundingClientRect().height };
            grip.setPointerCapture(event.pointerId);
        });
        grip.addEventListener("pointermove", event => {
            if (drag && grip.hasPointerCapture(event.pointerId)) resize(drag.height + event.clientY - drag.y);
        });
        grip.addEventListener("pointerup", event => { if (grip.hasPointerCapture(event.pointerId)) grip.releasePointerCapture(event.pointerId); });
        grip.addEventListener("lostpointercapture", () => { drag = undefined; });
        grip.addEventListener("keydown", event => {
            if (event.key !== "ArrowUp" && event.key !== "ArrowDown") return;
            event.preventDefault();
            resize(surface.getBoundingClientRect().height + (event.key === "ArrowUp" ? -24 : 24));
        });
        surface.append(grip);
        host.classList.add("cm-ready");
        grip.setAttribute("aria-valuenow", String(Math.round(surface.getBoundingClientRect().height)));
        const record = { view, issueLocations: [], issueIndex: -1, sync(value, requestedLanguage, label) {
            const nextLabel = label || input.getAttribute("aria-label") || "Code";
            const relabeled = nextLabel !== accessibleLabel;
            accessibleLabel = nextLabel;
            grip.setAttribute("aria-label", `Resize ${accessibleLabel} editor`);
            const next = value ?? view.state.doc.toString();
            const nextMode = languageName(requestedLanguage, next);
            const effects = mode === nextMode ? [] : [language.reconfigure(languages[nextMode] || [])];
            mode = nextMode;
            const replaced = value !== null && value !== view.state.doc.toString();
            if (replaced || effects.length || relabeled) {
                external = true;
                try {
                    if (replaced) view.setState(state(value));
                    else view.dispatch({ effects });
                }
                finally { external = false; }
            }
            if (replaced) {
                record.issueLocations = [];
                record.issueIndex = -1;
                updateIssueControls(host, record);
            }
        }, toggleWrap() {
            wrapped = !wrapped;
            view.dispatch({ effects: wrapping.reconfigure(wrapped ? EditorView.lineWrapping : []) });
            const button = host.querySelector(".editor-wrap-toggle");
            button?.setAttribute("aria-pressed", String(wrapped));
            button?.setAttribute("title", wrapped ? "Disable word wrap" : "Enable word wrap");
        } };
        editors.set(host, record);
        const pending = window.acornPendingEditorReveal;
        if (pending?.id === input.id) {
            delete window.acornPendingEditorReveal;
            queueMicrotask(() => window.acornEditors?.reveal(input.id, pending.line, pending.character, pending.endLine, pending.endCharacter));
        }
    }
    function discover() {
        for (const [host, editor] of editors) {
            if (!host.isConnected) {
                if (host.classList.contains("is-fullscreen")) document.body.classList.remove("editor-fullscreen-open");
                editor.view.destroy(); editors.delete(host);
            } else updateIssueControls(host, editor);
        }
        document.querySelectorAll(".code-editor").forEach(host => {
            mount(host);
            popouts.refresh(host);
            if (popouts.isPopout(host.querySelector("textarea")?.id) && !host.classList.contains("is-fullscreen")) setFullscreen(host, true);
        });
    }
    function find(id) { return editors.get(document.getElementById(id)?.closest(".code-editor")); }
    window.acornEditors = {
        sync(id, value, language, label) {
            discover();
            find(id)?.sync(value, language, label);
            if (value !== null) document.getElementById(id)?.dispatchEvent(new CustomEvent("acorn-editor-sync", { bubbles: true, detail: value }));
        },
        setDiagnostics(id, diagnostics, source) {
            discover();
            const editor = find(id);
            const view = editor?.view;
            if (!view || view.state.doc.toString() !== source) return;
            const grouped = new Map();
            const discovered = [];
            const discoveries = [];
            const lines = new Map();
            const locations = new Map();
            for (const diagnostic of diagnostics) {
                if (diagnostic.severity !== "error" && diagnostic.severity !== "warning" && diagnostic.severity !== "discovered") continue;
                const from = sourcePosition(view.state.doc, diagnostic.line, diagnostic.character);
                const to = sourcePosition(view.state.doc, diagnostic.endLine, diagnostic.endCharacter);
                if (from === null) continue;
                if (diagnostic.severity === "discovered") {
                    if (to !== null && from < to) {
                        const kind = typeof diagnostic.kind === "string" ? diagnostic.kind.toLowerCase() : "";
                        discovered.push(Decoration.mark({ class: "cm-discovered-pid", attributes: { "data-pid": kind } }).range(from, to));
                        discoveries.push({ from, to, severity: "discovered", kind, normalized: diagnostic.normalized, line: diagnostic.line });
                    }
                    continue;
                }
                const locationKey = `${from}:${to ?? from}`;
                if (!locations.has(locationKey)) locations.set(locationKey, {
                    from, to, line: diagnostic.line, character: diagnostic.character,
                    endLine: diagnostic.endLine, endCharacter: diagnostic.endCharacter,
                    severity: diagnostic.severity, messages: [],
                });
                const location = locations.get(locationKey);
                if (diagnostic.severity === "error") location.severity = "error";
                if (typeof diagnostic.message === "string" && !location.messages.includes(diagnostic.message)) location.messages.push(diagnostic.message);
                const lastLine = view.state.doc.lineAt(to !== null && to > from ? to - 1 : from).number;
                for (let number = diagnostic.line; number <= lastLine; number++) {
                    const offset = view.state.doc.line(number).from;
                    if (diagnostic.severity === "error" || !lines.has(offset)) lines.set(offset, diagnostic.severity);
                }
                if (to === null || from >= to) continue;
                const key = `${from}:${to}`;
                if (!grouped.has(key)) grouped.set(key, { from, to, line: diagnostic.line, path: diagnostic.path, severity: diagnostic.severity, messages: new Set() });
                const issue = grouped.get(key);
                if (diagnostic.severity === "error") issue.severity = "error";
                if (typeof diagnostic.message === "string") issue.messages.add(diagnostic.message);
            }
            const issues = Array.from(grouped.values()).sort((left, right) => (left.to - left.from) - (right.to - right.from) || left.from - right.from);
            const marks = issues.map(({ from, to, severity }) => Decoration.mark({ class: `cm-diagnostic cm-diagnostic-${severity}` }).range(from, to));
            const markers = Array.from(lines).sort(([left], [right]) => left - right).map(([offset, severity]) => diagnosticLineMarkers[severity].range(offset));
            view.dispatch({ effects: setDiagnosticMarks.of({ marks: Decoration.set([...marks, ...discovered], true), lines: RangeSet.of(markers), issues, discoveries }) });
            const active = editor.issueLocations[editor.issueIndex];
            editor.issueLocations = Array.from(locations.values()).sort((left, right) => left.from - right.from || (left.to ?? left.from) - (right.to ?? right.from));
            editor.issueIndex = active ? editor.issueLocations.findIndex(location => location.from === active.from && location.to === active.to) : -1;
            updateIssueControls(view.dom.closest(".code-editor"), editor);
        },
        reveal(id, lineNumber, characterNumber, endLineNumber, endCharacterNumber, center = false) {
            discover();
            const editor = find(id);
            if (!editor) return;
            const view = editor.view;
            const anchor = sourcePosition(view.state.doc, lineNumber, characterNumber);
            if (anchor === null) return;
            unfoldAll(view);
            const line = view.state.doc.line(lineNumber);
            const head = sourcePosition(view.state.doc, endLineNumber, endCharacterNumber) ?? Math.min(line.to, anchor + (Array.from(line.text)[characterNumber - 1]?.length || 0));
            view.dispatch({ selection: { anchor, head }, ...(center ? { effects: EditorView.scrollIntoView(anchor, { y: "center" }) } : { scrollIntoView: true }) });
            view.focus();
            editor.issueIndex = editor.issueLocations.findIndex(location => location.from === anchor && (location.to === null || location.to === head));
            updateIssueControls(view.dom.closest(".code-editor"), editor);
        },
        navigateIssue(id, direction) {
            discover();
            const editor = find(id);
            if (!editor?.issueLocations.length) return;
            const controls = editor.view.dom.closest(".code-editor")?.querySelector(".editor-issue-nav");
            const activeControl = controls?.contains(document.activeElement) ? document.activeElement : null;
            const count = editor.issueLocations.length;
            const displayedIndex = editor.issueIndex < 0 && controls?.closest(".is-fullscreen") ? 0 : editor.issueIndex;
            const index = displayedIndex < 0 ? (direction < 0 ? count - 1 : 0) : (displayedIndex + direction + count) % count;
            const issue = editor.issueLocations[index];
            window.acornEditors.reveal(id, issue.line, issue.character, issue.endLine, issue.endCharacter, true);
            editor.issueIndex = index;
            updateIssueControls(editor.view.dom.closest(".code-editor"), editor);
            activeControl?.focus({ preventScroll: true });
            requestAnimationFrame(() => {
                if (editor.issueIndex !== index || editor.view.dom.closest(".code-editor")?.classList.contains("is-fullscreen")) return;
                const rect = editor.view.coordsAtPos(issue.from);
                if (!rect) return;
                const viewport = window.visualViewport;
                const middle = viewport ? viewport.offsetTop + viewport.height / 2 : window.innerHeight / 2;
                window.scrollBy(0, (rect.top + rect.bottom) / 2 - middle);
            });
        },
        toggleFullscreen(id) {
            discover();
            const host = document.getElementById(id)?.closest(".code-editor");
            const view = find(id)?.view;
            if (!host || !view || popouts.isPopout(id)) return;
            setFullscreen(host, !host.classList.contains("is-fullscreen"));
            view.focus();
        },
        openInNewTab(id, state) {
            if (find(id)) popouts.open(id, state);
        },
        syncPopoutInput(id, value) { popouts.sendInput(id, value); },
        fold(id) { const editor = find(id); if (editor) { foldDocument(editor.view); editor.view.focus(); } },
        expand(id) { const editor = find(id); if (editor) { unfoldAll(editor.view); editor.view.focus(); } },
        openFind(id) { const editor = find(id); if (editor) openSearchPanel(editor.view); },
        setFontSize(id, size) {
            const host = document.getElementById(id)?.closest(".code-editor");
            if (!host || !["small", "big"].includes(size)) return;
            host.dataset.fontSize = size;
            host.querySelectorAll(".editor-font-size button").forEach(button => button.setAttribute("aria-pressed", String(button.dataset.size === size)));
            find(id)?.view.requestMeasure();
        },
        toggleWrap(id) { find(id)?.toggleWrap(); },
    };
    new MutationObserver((records) => {
        if (records.some(({ target }) => !target.closest?.(".cm-editor"))) discover();
    }).observe(document.documentElement, { childList: true, subtree: true });
    document.addEventListener("keydown", event => {
        if (event.key.toLowerCase() === "s" && (event.ctrlKey || event.metaKey) && !event.altKey && !event.shiftKey) {
            const save = event.target.closest?.(".code-editor")?.querySelector(".editor-save-action");
            if (save) {
                event.preventDefault();
                event.stopPropagation();
                if (!event.repeat && !save.disabled) save.click();
            }
        }
        if (event.key.toLowerCase() === "r" && event.ctrlKey && !event.altKey && !event.metaKey && !event.shiftKey) {
            const run = document.querySelector(".code-editor.is-fullscreen .editor-run-action");
            if (run) {
                event.preventDefault();
                event.stopPropagation();
                if (!event.repeat && !run.disabled) run.click();
            }
            return;
        }
        if ((event.key === "ArrowLeft" || event.key === "ArrowRight") && !event.altKey && !event.ctrlKey && !event.metaKey && !event.shiftKey) {
            const controls = event.target.closest?.(".editor-issue-nav:not([hidden])");
            const id = controls?.closest(".code-editor")?.querySelector("textarea")?.id;
            if (id) {
                event.preventDefault();
                window.acornEditors.navigateIssue(id, event.key === "ArrowLeft" ? -1 : 1);
            }
            return;
        }
        if (event.key !== "Escape") return;
        const host = document.querySelector(".code-editor.is-fullscreen");
        const id = host?.querySelector("textarea")?.id;
        if (!host || popouts.isPopout(id)) return;
        event.preventDefault();
        event.stopPropagation();
        window.acornEditors.toggleFullscreen(id);
    }, true);
    discover();
    window.dispatchEvent(new Event("acorn-editors-ready"));
}
if (typeof document !== "undefined") installEditors();
