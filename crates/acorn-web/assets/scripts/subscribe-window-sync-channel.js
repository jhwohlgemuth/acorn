// SPDX-License-Identifier: MIT
if (!window.acornWindowSync) {
    await new Promise(resolve => window.addEventListener('acorn-window-sync-ready', resolve, { once: true }));
}
const unsubscribe = window.acornWindowSync.subscribe(__acornArgs, message => dioxus.send(message));
await dioxus.recv();
unsubscribe();
