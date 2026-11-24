// SPDX-License-Identifier: MIT
if (!window.acornRemote) {
    await new Promise(resolve => window.addEventListener('acorn-remote-ready', resolve, { once: true }));
}
const [url, limit, workspaceFile] = __acornArgs;
try {
    return { ok: await window.acornRemote.file.load(url, limit, workspaceFile) };
} catch (error) {
    return { error: error.message };
}
