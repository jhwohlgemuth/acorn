// SPDX-License-Identifier: MIT
if (!window.acornWindowSync) {
    await new Promise(resolve => window.addEventListener('acorn-window-sync-ready', resolve, { once: true }));
}
const unsubscribe = window.acornWindowSync.subscribe(__acornArgs, message => dioxus.send(message));
const onRefresh = event => {
    if (event.origin !== location.origin || event.source?.opener !== window) return;
    if (event.data?.type === 'acorn-editor-state-refresh' && event.data.topic === __acornArgs && Number.isSafeInteger(event.data.after)) {
        dioxus.send({ type: 'refresh', after: event.data.after });
    }
};
window.addEventListener('message', onRefresh);
if (window.name.startsWith('acorn-editor:')) {
    try {
        const [topic, message] = JSON.parse(window.sessionStorage.getItem('acorn.editor.handoff.v1'));
        if (topic === __acornArgs && message?.type === 'state') {
            window.sessionStorage.removeItem('acorn.editor.handoff.v1');
            dioxus.send(message);
        }
    } catch {}
}
await dioxus.recv();
window.removeEventListener('message', onRefresh);
unsubscribe();
