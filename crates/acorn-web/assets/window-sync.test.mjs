import assert from "node:assert/strict";
import { test } from "node:test";
import { createWindowSync } from "./window-sync.js";

const connections = new Map();
class FakeBroadcastChannel {
    constructor(topic) {
        this.topic = topic;
        if (!connections.has(topic)) connections.set(topic, new Set());
        connections.get(topic).add(this);
    }
    postMessage(data) {
        for (const peer of connections.get(this.topic)) {
            if (peer !== this) peer.onmessage?.({ data: structuredClone(data) });
        }
    }
    close() { connections.get(this.topic).delete(this); }
}
let nextId = 0;
function environment(origin = "https://acorn.example") {
    const listeners = new Map();
    const stored = new Map();
    return {
        BroadcastChannel: FakeBroadcastChannel,
        crypto: { randomUUID: () => String(++nextId) },
        location: { origin },
        localStorage: { setItem: (key, value) => stored.set(key, value) },
        addEventListener: (type, listener) => {
            if (!listeners.has(type)) listeners.set(type, new Set());
            listeners.get(type).add(listener);
        },
        removeEventListener: (type, listener) => listeners.get(type)?.delete(listener),
        dispatch(type, event) { for (const listener of listeners.get(type) ?? []) listener(event); },
        stored,
    };
}
test("broadcast subscriptions receive peer messages by topic and stop after unsubscribe", () => {
    const left = createWindowSync(environment());
    const right = createWindowSync(environment());
    const received = [];
    const stopLeft = left.subscribe("tool", message => received.push(["left", message]));
    const stopRight = right.subscribe("tool", message => received.push(["right", message]));
    left.publish("tool", { input: "one" });
    right.publish("layout", { input: "ignored" });
    right.publish("tool", { input: "two" });
    stopRight();
    left.publish("tool", { input: "three" });
    stopLeft();
    assert.deepEqual(received, [["right", { input: "one" }], ["left", { input: "two" }]]);
});
test("direct messages are origin checked and storage listeners match their key", () => {
    const source = environment();
    const peer = environment();
    const sync = createWindowSync(source);
    const messages = [];
    source.opener = { postMessage: (payload, origin) => messages.push([payload, origin]) };
    sync.postToOpener({ type: "opened" });
    const stop = sync.onDirect(event => messages.push(event.data));
    source.dispatch("message", { origin: "https://elsewhere.example", data: "ignored" });
    source.dispatch("message", { origin: source.location.origin, data: "accepted" });
    stop();
    source.dispatch("message", { origin: source.location.origin, data: "ignored after stop" });
    const storedChanges = [];
    const stopStorage = sync.subscribeStorage("workspace", value => storedChanges.push(value));
    source.dispatch("storage", { key: "layout" });
    source.dispatch("storage", { key: "workspace", newValue: "updated" });
    stopStorage();
    source.dispatch("storage", { key: "workspace", newValue: "ignored" });
    sync.notifyStorage("workspace");
    assert.ok(source.stored.has("workspace"));
    assert.deepEqual(storedChanges, ["updated"]);
    assert.deepEqual(messages, [[{ type: "opened" }, source.location.origin], "accepted"]);
    assert.equal(peer.stored.size, 0);
});
