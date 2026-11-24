// SPDX-License-Identifier: MIT
if (!window.acornMap) {
    await new Promise(resolve => window.addEventListener('acorn-effects-ready', resolve, { once: true }));
}
const [module, css] = __acornArgs;
await window.acornMap.connect(module, css, event => dioxus.send(event));
await dioxus.recv();
