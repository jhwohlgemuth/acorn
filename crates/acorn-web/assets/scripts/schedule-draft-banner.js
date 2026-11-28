// SPDX-License-Identifier: MIT
const state = document.__draftBannerState || (document.__draftBannerState = {});
clearTimeout(state.pending);
clearTimeout(state.hide);
const startProgress = (banner, duration) => {
    if (state.progress) state.progress.cancel();
    state.progress = null;
    const bar = banner.querySelector('.editor-draft-progress');
    if (bar && !matchMedia('(prefers-reduced-motion: reduce)').matches) {
        state.progress = bar.animate([{ transform: 'scaleX(1)' }, { transform: 'scaleX(0)' }], { duration, fill: 'forwards' });
    }
};
const banner = document.querySelector('.editor-draft');
if (banner) startProgress(banner, 10000);
state.hide = setTimeout(() => {
    const current = document.querySelector('.editor-draft');
    if (current) current.classList.remove('is-visible');
}, 10000);
state.pending = setTimeout(() => {
    const current = document.querySelector('.editor-draft');
    if (current) {
        current.classList.add('is-visible');
        if (!banner) startProgress(current, 9700);
    }
}, 300);
