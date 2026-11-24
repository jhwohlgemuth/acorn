// SPDX-License-Identifier: MIT
if (!window.acornUrl) {
    await new Promise(resolve => window.addEventListener('acorn-url-ready', resolve, { once: true }));
}
if (!window.acornWindowSync) {
    await new Promise(resolve => window.addEventListener('acorn-window-sync-ready', resolve, { once: true }));
}
const initial = window.acornInitialReviewFragment;
if (initial && !window.acornUrl.getFragment()) window.acornUrl.setFragment(initial, true);
window.acornInitialReviewFragment = '';
const receive = window.acornWindowSync.subscribe('acorn.review.mode.v1', fragment => {
    if (typeof fragment === 'string' && fragment !== window.acornUrl.getFragment()) {
        window.acornUrl.setFragment(fragment, true);
    }
});
const unsubscribe = window.acornUrl.subscribe(fragment => {
    window.acornWindowSync.publish('acorn.review.mode.v1', fragment);
    dioxus.send(fragment);
});
await dioxus.recv();
unsubscribe();
receive();
