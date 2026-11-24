// SPDX-License-Identifier: MIT
window.acornChrome?.tour.cancel();
window.acornChrome?.theme.sync();
requestAnimationFrame(() => {
    const heading = document.querySelector('#content-main h1');
    if (heading) {
        heading.tabIndex = -1;
        heading.focus();
    }
    window.scrollTo(0, 0);
});
