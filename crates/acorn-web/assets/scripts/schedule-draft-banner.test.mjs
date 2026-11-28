import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { runInNewContext } from "node:vm";
import test from "node:test";

const source = await readFile(new URL("./schedule-draft-banner.js", import.meta.url), "utf8");

test("draft banner closes ten seconds after the latest edit", () => {
    let now = 0;
    let nextId = 0;
    const timers = new Map();
    const classes = new Set();
    const bar = { animate: () => ({ cancel() {} }) };
    const banner = {
        classList: {
            add: (name) => classes.add(name),
            remove: (name) => classes.delete(name),
        },
        querySelector: () => bar,
    };
    const document = { querySelector: () => banner };
    const setTimeout = (callback, delay) => {
        const id = ++nextId;
        timers.set(id, { at: now + delay, callback });
        return id;
    };
    const clearTimeout = (id) => timers.delete(id);
    const advance = (duration) => {
        const target = now + duration;
        const due = () => [...timers].filter(([, timer]) => timer.at <= target).sort((a, b) => a[1].at - b[1].at)[0];
        for (let task = due(); task; task = due()) {
            timers.delete(task[0]);
            now = task[1].at;
            task[1].callback();
        }
        now = target;
    };
    const edit = () => runInNewContext(source, { document, setTimeout, clearTimeout, matchMedia: () => ({ matches: false }) });

    edit();
    advance(100);
    edit();
    advance(299);
    assert.equal(classes.has("is-visible"), false);
    advance(1);
    assert.equal(classes.has("is-visible"), true);

    advance(9000);
    edit();
    advance(300);
    edit();
    advance(300);
    edit();
    advance(9999);
    assert.equal(classes.has("is-visible"), true);
    advance(1);
    assert.equal(classes.has("is-visible"), false);
});
