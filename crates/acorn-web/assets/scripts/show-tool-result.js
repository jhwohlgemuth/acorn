// SPDX-License-Identifier: MIT
const [slug, celebrate] = __acornArgs;
requestAnimationFrame(() => {
    const fullscreen = document.querySelector('.code-editor.is-fullscreen');
    const section = [...document.querySelectorAll('[data-tool]')]
        .find(element => element.dataset.tool === slug);
    const heading = section?.querySelector('#result-heading');
    if (fullscreen) fullscreen.querySelector('.editor-run-action')?.focus({ preventScroll: true });
    else heading?.focus();
    if (heading && celebrate) window.acornCelebrate?.();
});
