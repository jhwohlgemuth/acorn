function escapeCode(text) {
    return text.replaceAll("&", "&amp;").replaceAll("<", "&lt;").replaceAll(">", "&gt;");
}
function highlightCode(Prism, text, language) {
    // Bound synchronous grammar work, including Markdown's costly long-line matching.
    if (text.length > 16_384 || text.split(/[\r\n]/).some((line) => line.length > 512)) return escapeCode(text);
    const aliases = { "json-ld": "json", jsonc: "javascript", cff: "yaml", "rdf-xml": "markup", xml: "markup", zon: "zig", text: "markdown" };
    const selected = aliases[language] || language;
    const detected = selected === "auto" ? (/^\s*[[{]/.test(text) ? "json" : /^\s*</.test(text) ? "markup" : "markdown") : selected;
    const grammar = Prism.languages[detected] || Prism.languages.markdown;
    return Prism.highlight(text, grammar, detected);
}
function lineStarts(text) {
    return [0, ...Array.from(text.matchAll(/\r\n|[\r\n]/g), (match) => match.index + match[0].length).filter((offset) => offset < text.length)];
}
function lowerBound(length, before) {
    let low = 0;
    let high = length;
    while (low < high) {
        const middle = Math.floor((low + high) / 2);
        if (before(middle)) low = middle + 1;
        else high = middle;
    }
    return low;
}
function observeLineNumbers() {
    const hosts = new Map();
    const selector = ".code-viewport";
    function attach(host) {
        if (hosts.has(host)) return;
        const pre = host.querySelector("pre");
        const gutter = host.querySelector(".line-numbers");
        if (!pre || !gutter) return;
        let dirty = true;
        let frame = 0;
        let starts = [];
        let nodes = [];
        function render() {
            frame = 0;
            if (!pre.clientHeight || !host.isConnected) return;
            if (dirty) {
                starts = lineStarts(pre.textContent);
                nodes = [];
                const walker = document.createTreeWalker(pre, NodeFilter.SHOW_TEXT);
                let end = 0;
                while (walker.nextNode()) {
                    const node = walker.currentNode;
                    end += node.length;
                    if (node.length) nodes.push({ node, end });
                }
                dirty = false;
            }
            const bounds = pre.getBoundingClientRect();
            const origin = gutter.getBoundingClientRect().top;
            const style = getComputedStyle(pre);
            const height = parseFloat(style.lineHeight);
            const range = document.createRange();
            function top(index) {
                const offset = starts[index];
                const part = nodes[lowerBound(nodes.length, (index) => nodes[index].end <= offset)];
                if (!part) return bounds.top + pre.clientTop + parseFloat(style.paddingTop) - pre.scrollTop;
                const position = offset - (part.end - part.node.length);
                range.setStart(part.node, position);
                range.setEnd(part.node, position + 1);
                const rect = range.getBoundingClientRect();
                return rect.top - (height - rect.height) / 2;
            }
            // Measure only visible logical lines; wrapped continuations keep the same number.
            const first = lowerBound(starts.length, (index) => top(index) + height < bounds.top);
            const visible = document.createDocumentFragment();
            const bottom = bounds.top + pre.clientTop + pre.clientHeight;
            const limit = Math.ceil(pre.clientHeight / height) + 2;
            for (let index = first; index < starts.length && index < first + limit; index += 1) {
                const y = top(index);
                if (y >= bottom) break;
                const number = document.createElement("span");
                number.dataset.line = String(index + 1);
                number.style.top = `${y - origin}px`;
                visible.append(number);
            }
            gutter.replaceChildren(visible);
        }
        function schedule() {
            if (!frame) frame = requestAnimationFrame(render);
        }
        const resize = new ResizeObserver(schedule);
        resize.observe(pre);
        pre.addEventListener("scroll", schedule, { passive: true });
        const content = new MutationObserver(() => { dirty = true; schedule(); });
        content.observe(pre, { characterData: true, childList: true, subtree: true });
        hosts.set(host, () => {
            cancelAnimationFrame(frame);
            content.disconnect();
            resize.disconnect();
            pre.removeEventListener("scroll", schedule);
        });
        schedule();
        document.fonts.ready.then(schedule);
    }
    function discover() {
        for (const [host, dispose] of hosts) {
            if (!host.isConnected) { dispose(); hosts.delete(host); }
        }
        document.querySelectorAll(selector).forEach(attach);
    }
    new MutationObserver((records) => {
        if (records.some(({ target }) => !target.closest?.(".line-numbers"))) discover();
    }).observe(document.documentElement, { childList: true, subtree: true });
    discover();
}
if (typeof document !== "undefined") observeLineNumbers();
if (typeof document !== "undefined" && document.currentScript?.dataset.prism) {
    import(document.currentScript.dataset.prism).then(() => {
        Prism.languages.expression = { function: /\b(?:O|log|ln|exp|sqrt)\b/, number: /\b\d+(?:\.\d+)?\b/, operator: /[+*/^=−-]/, punctuation: /[(),]/, variable: /\b[a-zA-Z]\b/ };
        Prism.languages.identifier = { keyword: /\b(?:https?|doi|orcid|ror|ark|urn)(?=:)/i, number: /\d+/, punctuation: /[:/.#-]/ };
        Prism.languages.prompt = { variable: /\$[\w-]+/, string: /`[^`]+`/ };
        window.acornSyntax = { highlight: (text, language) => highlightCode(Prism, text, language) };
    }).catch(() => {
        window.acornSyntax = { highlight: escapeCode };
    }).finally(() => window.dispatchEvent(new Event("acorn-syntax-ready")));
}
