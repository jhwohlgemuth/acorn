// SPDX-License-Identifier: MIT
if (!window.acornRemote) {
    await new Promise(resolve => window.addEventListener('acorn-remote-ready', resolve, { once: true }));
}
const [url, reference] = __acornArgs;
let result;
try {
    result = { ok: await window.acornRemote.load(url, reference) };
} catch (error) {
    result = { error: error.message };
}
return { ...result, history: window.acornRemote.history.read() };
