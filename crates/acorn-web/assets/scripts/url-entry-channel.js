// SPDX-License-Identifier: MIT
if (!window.acornUrl) {
    await new Promise(resolve => window.addEventListener('acorn-url-ready', resolve, { once: true }));
}
const keys = ['code', 'template', 'url'];
const unsubscribe = window.acornUrl.subscribe(() => {
    const parameters = Object.fromEntries(keys.map(key => [key, window.acornUrl.getParameters(key)]));
    dioxus.send({
        href: window.acornUrl.getHref(),
        parameters,
        malformed: keys.some(key => window.acornUrl.hasMalformedEncoding(key)),
    });
});
await dioxus.recv();
unsubscribe();
