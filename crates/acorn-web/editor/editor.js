import { defaultKeymap, history, historyKeymap } from "@codemirror/commands";
import { javascript } from "@codemirror/lang-javascript";
import { json } from "@codemirror/lang-json";
import { markdown } from "@codemirror/lang-markdown";
import { xml } from "@codemirror/lang-xml";
import { yaml } from "@codemirror/lang-yaml";
import { bracketMatching, foldable, foldAll, foldEffect, foldGutter, foldKeymap, forceParsing, indentOnInput, StreamLanguage, syntaxHighlighting, unfoldAll } from "@codemirror/language";
import { toml } from "@codemirror/legacy-modes/mode/toml";
import { turtle } from "@codemirror/legacy-modes/mode/turtle";
import { Compartment, EditorState } from "@codemirror/state";
import { drawSelection, EditorView, highlightActiveLine, highlightActiveLineGutter, keymap, lineNumbers } from "@codemirror/view";
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
function installEditors() {
    const editors = new Map();
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
            lineNumbers(), foldGutter(), history(), drawSelection(), bracketMatching(), indentOnInput(),
            highlightActiveLine(), highlightActiveLineGutter(), EditorView.lineWrapping,
            syntaxHighlighting(classHighlighter),
            keymap.of([...foldKeymap.map(binding => binding.run === foldAll ? { ...binding, run: foldDocument } : binding), ...defaultKeymap, ...historyKeymap]),
            EditorView.contentAttributes.of(() => ({ id: `${input.id}-editor`, "aria-label": accessibleLabel, "aria-readonly": String(input.readOnly), tabindex: "0", spellcheck: "false", autocapitalize: "off" })),
            EditorView.updateListener.of((update) => {
                if (!update.docChanged || external) return;
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
        const record = { view, sync(value, requestedLanguage, label) {
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
        } };
        editors.set(host, record);
    }
    function discover() {
        for (const [host, { view }] of editors) {
            if (!host.isConnected) { view.destroy(); editors.delete(host); }
        }
        document.querySelectorAll(".code-editor").forEach(mount);
    }
    function find(id) { return editors.get(document.getElementById(id)?.closest(".code-editor")); }
    window.acornEditors = {
        sync(id, value, language, label) { discover(); find(id)?.sync(value, language, label); },
        fold(id) { const editor = find(id); if (editor) { foldDocument(editor.view); editor.view.focus(); } },
        expand(id) { const editor = find(id); if (editor) { unfoldAll(editor.view); editor.view.focus(); } },
    };
    new MutationObserver((records) => {
        if (records.some(({ target }) => !target.closest?.(".cm-editor"))) discover();
    }).observe(document.documentElement, { childList: true, subtree: true });
    discover();
}
if (typeof document !== "undefined") installEditors();
