// SPDX-License-Identifier: MIT
const [line, character, endLine, endCharacter] = __acornArgs;
const tab = document.getElementById('details-tab-source');
if (tab?.closest('dialog')?.open) {
    window.acornPendingEditorReveal = { id: 'drawer-source', line: line, character: character, endLine: endLine, endCharacter: endCharacter };
    tab.click();
} else {
    window.acornEditors?.reveal('content', line, character, endLine, endCharacter);
}
