// SPDX-License-Identifier: MIT
if (!window.acornWindowSync) {
    await new Promise(resolve => window.addEventListener('acorn-window-sync-ready', resolve, { once: true }));
}
const unsubscribe = window.acornWindowSync.subscribeStorage(__acornArgs, () => dioxus.send(true));
await dioxus.recv();
unsubscribe();
