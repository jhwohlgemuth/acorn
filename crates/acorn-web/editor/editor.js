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
import { Decoration, drawSelection, EditorView, GutterMarker, highlightActiveLine, highlightActiveLineGutter, hoverTooltip, keymap, lineNumberMarkers, lineNumbers } from "@codemirror/view";
import { classHighlighter } from "@lezer/highlight";

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
const emptyDiagnostics = { marks: Decoration.none, lines: RangeSet.empty, issues: [] };
class DiagnosticLineMarker extends GutterMarker { elementClass = "cm-diagnostic-line"; }
const diagnosticLineMarker = new DiagnosticLineMarker();
const diagnosticMarks = StateField.define({
    create: () => emptyDiagnostics,
    update(diagnostics, transaction) {
        if (transaction.docChanged) return emptyDiagnostics;
        return transaction.effects.find(effect => effect.is(setDiagnosticMarks))?.value ?? diagnostics;
    },
    provide: field => [EditorView.decorations.from(field, value => value.marks), lineNumberMarkers.from(field, value => value.lines)],
});
function diagnosticTooltip(issue) {
    const dom = document.createElement("div");
    dom.className = "cm-diagnostic-tooltip";
    dom.dataset.severity = issue.severity;
    dom.setAttribute("role", "tooltip");
    const heading = document.createElement("div");
    heading.className = "cm-diagnostic-tooltip-heading";
    const badge = document.createElement("strong");
    badge.className = "cm-diagnostic-tooltip-badge";
    badge.textContent = issue.severity === "error" ? "Error" : "Warning";
    const location = document.createElement("span");
    location.textContent = `Line ${issue.line}`;
    heading.append(badge, location);
    dom.append(heading);
    if (issue.path) {
        const path = document.createElement("div");
        path.className = "cm-diagnostic-tooltip-path";
        path.textContent = issue.path;
        dom.append(path);
    }
    for (const message of issue.messages) {
        const body = document.createElement("p");
        body.textContent = message;
        dom.append(body);
    }
    return { dom };
}
const diagnosticHover = hoverTooltip((view, pos) => {
    const issue = view.state.field(diagnosticMarks).issues.find(({ from, to }) => from <= pos && pos <= to);
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
    function updateIssueControls(host, editor) {
        const controls = host.querySelector(".editor-issue-nav");
        if (!controls) return;
        const count = editor.issueLocations.length;
        controls.hidden = count === 0;
        const position = controls.querySelector(".editor-issue-position");
        const label = `${editor.issueIndex + 1} of ${count}`;
        if (position && position.textContent !== label) position.textContent = label;
    }
    function mount(host) {
        if (editors.has(host)) return;
        const input = host.querySelector("textarea");
        const surface = host.querySelector(".editor-surface");
        if (!input || !surface) return;
        const language = new Compartment();
        let accessibleLabel = input.getAttribute("aria-label") || "Code";
        let mode = languageName(host.dataset.language, input.value);
        let external = false;
        const extensions = [
            EditorState.readOnly.of(input.readOnly), EditorView.editable.of(!input.readOnly),
            lineNumbers(), foldGutter(), history(), drawSelection(), diagnosticMarks, diagnosticHover, bracketMatching(), indentOnInput(),
            highlightActiveLine(), highlightActiveLineGutter(), EditorView.lineWrapping,
            syntaxHighlighting(classHighlighter),
            keymap.of([...foldKeymap.map(binding => binding.run === foldAll ? { ...binding, run: foldDocument } : binding), ...defaultKeymap, ...historyKeymap]),
            EditorView.contentAttributes.of(() => ({ id: `${input.id}-editor`, "aria-label": accessibleLabel, "aria-readonly": String(input.readOnly), tabindex: "0", spellcheck: "false", autocapitalize: "off" })),
            EditorView.updateListener.of((update) => {
                if (!update.docChanged || external) return;
                const editor = editors.get(host);
                if (editor) {
                    editor.issueLocations = [];
                    editor.issueIndex = -1;
                    updateIssueControls(host, editor);
                }
                input.value = update.state.doc.toString();
                queueMicrotask(() => input.dispatchEvent(new Event("input", { bubbles: true })));
            }),
        ];
        const state = (text) => EditorState.create({ doc: text, extensions: [extensions, language.of(languages[mode] || [])] });
        const view = new EditorView({ parent: surface, state: state(input.value) });
        const grip = document.createElement("div");
        grip.className = "editor-resize-handle";
        grip.tabIndex = 0;
        grip.setAttribute("role", "separator");
        grip.setAttribute("aria-orientation", "horizontal");
        grip.setAttribute("aria-label", `Resize ${accessibleLabel} editor`);
        grip.setAttribute("aria-valuemin", "86");
        grip.setAttribute("aria-valuemax", "2048");
        grip.title = "Drag to resize · Up/Down arrows to adjust height";
        function resize(height) {
            const size = Math.round(Math.max(86, Math.min(2048, height)));
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
        document.querySelectorAll(".code-editor").forEach(mount);
    }
    function find(id) { return editors.get(document.getElementById(id)?.closest(".code-editor")); }
    window.acornEditors = {
        sync(id, value, language, label) { discover(); find(id)?.sync(value, language, label); },
        setDiagnostics(id, diagnostics, source) {
            discover();
            const editor = find(id);
            const view = editor?.view;
            if (!view || view.state.doc.toString() !== source) return;
            const grouped = new Map();
            const lines = new Set();
            const locations = new Map();
            for (const diagnostic of diagnostics) {
                if (diagnostic.severity !== "error" && diagnostic.severity !== "warning") continue;
                const from = sourcePosition(view.state.doc, diagnostic.line, diagnostic.character);
                const to = sourcePosition(view.state.doc, diagnostic.endLine, diagnostic.endCharacter);
                if (from === null) continue;
                const locationKey = `${from}:${to ?? from}`;
                if (!locations.has(locationKey)) locations.set(locationKey, {
                    from, to, line: diagnostic.line, character: diagnostic.character,
                    endLine: diagnostic.endLine, endCharacter: diagnostic.endCharacter,
                });
                const lastLine = view.state.doc.lineAt(to !== null && to > from ? to - 1 : from).number;
                for (let number = diagnostic.line; number <= lastLine; number++) lines.add(view.state.doc.line(number).from);
                if (to === null || from >= to) continue;
                const key = `${from}:${to}`;
                if (!grouped.has(key)) grouped.set(key, { from, to, line: diagnostic.line, path: diagnostic.path, severity: diagnostic.severity, messages: new Set() });
                const issue = grouped.get(key);
                if (diagnostic.severity === "error") issue.severity = "error";
                if (typeof diagnostic.message === "string") issue.messages.add(diagnostic.message);
            }
            const issues = Array.from(grouped.values()).sort((left, right) => (left.to - left.from) - (right.to - right.from) || left.from - right.from);
            const marks = issues.map(({ from, to }) => Decoration.mark({ class: "cm-diagnostic" }).range(from, to));
            const markers = Array.from(lines).sort((left, right) => left - right).map(offset => diagnosticLineMarker.range(offset));
            view.dispatch({ effects: setDiagnosticMarks.of({ marks: Decoration.set(marks, true), lines: RangeSet.of(markers), issues }) });
            const active = editor.issueLocations[editor.issueIndex];
            editor.issueLocations = Array.from(locations.values()).sort((left, right) => left.from - right.from || (left.to ?? left.from) - (right.to ?? right.from));
            editor.issueIndex = active ? editor.issueLocations.findIndex(location => location.from === active.from && location.to === active.to) : -1;
            updateIssueControls(view.dom.closest(".code-editor"), editor);
        },
        reveal(id, lineNumber, characterNumber, endLineNumber, endCharacterNumber) {
            discover();
            const editor = find(id);
            if (!editor) return;
            const view = editor.view;
            const anchor = sourcePosition(view.state.doc, lineNumber, characterNumber);
            if (anchor === null) return;
            unfoldAll(view);
            const line = view.state.doc.line(lineNumber);
            const head = sourcePosition(view.state.doc, endLineNumber, endCharacterNumber) ?? Math.min(line.to, anchor + (Array.from(line.text)[characterNumber - 1]?.length || 0));
            view.dispatch({ selection: { anchor, head }, scrollIntoView: true });
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
            const index = editor.issueIndex < 0 ? (direction < 0 ? count - 1 : 0) : (editor.issueIndex + direction + count) % count;
            const issue = editor.issueLocations[index];
            window.acornEditors.reveal(id, issue.line, issue.character, issue.endLine, issue.endCharacter);
            editor.issueIndex = index;
            updateIssueControls(editor.view.dom.closest(".code-editor"), editor);
            activeControl?.focus({ preventScroll: true });
        },
        toggleFullscreen(id) {
            discover();
            const host = document.getElementById(id)?.closest(".code-editor");
            const view = find(id)?.view;
            if (!host || !view) return;
            const active = host.classList.toggle("is-fullscreen");
            document.body.classList.toggle("editor-fullscreen-open", active);
            const button = host.querySelector(".editor-fullscreen-toggle");
            button?.setAttribute("aria-pressed", String(active));
            button?.setAttribute("title", active ? "Exit full screen (Esc)" : "Full screen (Esc to exit)");
            view.requestMeasure();
            view.focus();
        },
        fold(id) { const editor = find(id); if (editor) { foldDocument(editor.view); editor.view.focus(); } },
        expand(id) { const editor = find(id); if (editor) { unfoldAll(editor.view); editor.view.focus(); } },
    };
    new MutationObserver((records) => {
        if (records.some(({ target }) => !target.closest?.(".cm-editor"))) discover();
    }).observe(document.documentElement, { childList: true, subtree: true });
    document.addEventListener("keydown", event => {
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
        if (!host) return;
        event.preventDefault();
        event.stopPropagation();
        window.acornEditors.toggleFullscreen(host.querySelector("textarea")?.id);
    }, true);
    discover();
    window.dispatchEvent(new Event("acorn-editors-ready"));
}
if (typeof document !== "undefined") installEditors();
