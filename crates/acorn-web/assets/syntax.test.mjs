import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { runInNewContext } from "node:vm";
import test from "node:test";
const context = { window: {} };
runInNewContext(await readFile(new URL("vendor/prism.js", import.meta.url), "utf8"), context);
runInNewContext(await readFile(new URL("syntax.js", import.meta.url), "utf8"), context);
const highlight = (text, language) => context.highlightCode(context.Prism, text, language);
test("bounded highlighting still tokenizes content at the size and line limits", () => {
    const line = "x".repeat(511) + "\n";
    const text = line.repeat(32);
    let calls = 0;
    const prism = { languages: { markdown: {} }, highlight: (input) => { calls += 1; return input; } };
    assert.equal(context.highlightCode(prism, text, "text"), text);
    assert.equal(context.highlightCode(prism, "x".repeat(512), "text"), "x".repeat(512));
    assert.equal(calls, 2);
});
test("code highlighting escapes hostile HTML and preserves literal text", () => {
    const text = '{"title":"<script>alert(1)</script>","valid":true,"count":2}';
    const html = highlight(text, "json");
    assert.doesNotMatch(html, /<script>/);
    assert.match(html, /token property/);
    assert.match(html, /token boolean/);
    assert.match(html, /token number/);
    assert.equal(html.replace(/<[^>]*>/g, "").replaceAll("&lt;", "<").replaceAll("&gt;", ">").replaceAll("&amp;", "&"), text);
});
test("line offsets preserve blank lines and normalize newline boundaries", () => {
    for (const [text, expected] of [["", [0]], ["one", [0]], ["one\n", [0]], ["one\n\n", [0, 4]], ["\n\n\n", [0, 1, 2]], ["one\r\ntwo\rthree\nfour", [0, 5, 9, 15]]]) {
        assert.deepEqual(Array.from(context.lineStarts(text)), expected);
    }
});
test("oversized documents and long lines bypass Prism without losing or interpreting text", () => {
    const inputs = [
        "Some research words here. ".repeat(4923) + "\n",
        ("x".repeat(511) + "\n").repeat(32) + "x",
        "x".repeat(513),
        "short\r\n" + "x".repeat(513) + "\n",
        '<img src=x onerror="alert(1)">&'.repeat(30) + "\n",
    ];
    // No Prism object: any attempt to tokenize the fallback inputs fails this test.
    for (const text of inputs) {
        for (const language of ["auto", "json", "markdown", "text"]) {
            const html = context.highlightCode(null, text, language);
            assert.doesNotMatch(html, /<[^>]*>/);
            assert.equal(html.replaceAll("&lt;", "<").replaceAll("&gt;", ">").replaceAll("&amp;", "&"), text);
        }
    }
});
test("supported metadata and documentation grammars produce tokens", () => {
    for (const [language, text] of [["yaml", "title: Example\nvalid: true"], ["markdown", "# Title\n`code`"], ["rdf-xml", '<root id="1"/>'], ["diff", "-before\n+after\n"], ["jsonc", '{ /* note */ "x": 1 }'], ["turtle", '@prefix x: <https://example.org/> .'], ["zon", '.{ .name = "value" }']]) {
        assert.match(highlight(text, language), /class="token/, language);
    }
});
