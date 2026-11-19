import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";
import { runInNewContext } from "node:vm";

const context = { URL, Event, window: undefined };
runInNewContext(await readFile(new URL("url-state.js", import.meta.url), "utf8"), context);

function fixture(href = "https://example.org/acorn/review?topic=one#fair") {
    const events = new EventTarget();
    const location = { href };
    const history = {
        state: { selected: true },
        pushState(_state, _title, url) { location.href = url; },
        replaceState(_state, _title, url) { location.href = url; },
    };
    const environment = {
        location, history,
        addEventListener: events.addEventListener.bind(events),
        removeEventListener: events.removeEventListener.bind(events),
        dispatchEvent: events.dispatchEvent.bind(events),
    };
    return { state: context.createUrlState(environment), location, events };
}

test("URL state preserves path, unrelated parameters, and fragment", () => {
    const { state, location } = fixture();
    assert.deepEqual(Array.from(state.getParameters("topic")), ["one"]);
    assert.equal(state.setParameters("filter", ["A & B", "C"]), true);
    assert.deepEqual(new URL(location.href).searchParams.getAll("filter"), ["A & B", "C"]);
    assert.equal(state.getFragment(), "fair");
    assert.equal(state.setFragment("readability"), true);
    assert.equal(new URL(location.href).pathname, "/acorn/review");
    assert.equal(new URL(location.href).hash, "#readability");
    assert.equal(state.setFragment("readability"), false);
    assert.equal(state.setParameters("filter", null), true);
    assert.deepEqual(new URL(location.href).searchParams.getAll("filter"), []);
    assert.deepEqual(Array.from(state.getParameters("topic")), ["one"]);
});

test("subscribers receive initial state and browser history changes once", () => {
    const { state, location, events } = fixture();
    const seen = [];
    const unsubscribe = state.subscribe((fragment) => seen.push(fragment));
    state.setFragment("readability");
    events.dispatchEvent(new Event("hashchange"));
    location.href = "https://example.org/acorn/review?topic=one#fair";
    events.dispatchEvent(new Event("popstate"));
    unsubscribe();
    state.setFragment("");
    assert.deepEqual(seen, ["fair", "readability", "fair"]);
});

test("malformed fragments remain readable", () => {
    const { state } = fixture("https://example.org/acorn/review#%GG");
    assert.equal(state.getFragment(), "%GG");
});
