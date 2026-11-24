// SPDX-License-Identifier: MIT
const [id, replacement, language, label, diagnostics, source] = __acornArgs;
const versions = window.acornEditorSyncVersions ||= {};
const version = versions[id] = (versions[id] || 0) + 1;
if (!window.acornEditors) {
    await new Promise(resolve => window.addEventListener('acorn-editors-ready', resolve, { once: true }));
}
if (versions[id] === version && document.getElementById(id)?.value === source) {
    window.acornEditors.sync(id, replacement, language, label);
    window.acornEditors.setDiagnostics(id, diagnostics, source);
}
