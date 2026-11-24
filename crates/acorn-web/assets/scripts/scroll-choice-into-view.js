// SPDX-License-Identifier: MIT
requestAnimationFrame(() => {
    const bar = [...document.querySelectorAll('.choice-scroll')]
        .find(element => element.dataset.choiceGroup === __acornArgs);
    const selected = bar?.querySelector('[aria-pressed="true"]');
    if (selected) {
        bar.scrollLeft = selected.offsetLeft - (bar.clientWidth - selected.clientWidth) / 2;
    }
});
