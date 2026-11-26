import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { runInNewContext } from "node:vm";
import test from "node:test";

const source = await readFile(new URL("./ror-index-worker.js", import.meta.url), "utf8");

test("ROR worker sends only requested records and fetches without credentials", async () => {
    const messages = [];
    const requests = [];
    const index = {
        version: "fixture",
        date: "fixture",
        source: "fixture",
        sourceSha256: "fixture",
        license: "CC0-1.0",
        organizations: {
            "https://ror.org/01qz5mb56": { name: "Requested", locations: [] },
            "https://ror.org/04aj4c181": { name: "Unrequested", locations: [] },
        },
    };
    const bytes = new TextEncoder().encode(JSON.stringify(index));
    const self = { postMessage: (message) => messages.push(message) };
    runInNewContext(source, {
        self,
        TextDecoder,
        fetch: async (url, options) => {
            requests.push({ url, options });
            return { ok: true, arrayBuffer: async () => bytes.buffer };
        },
    });
    await self.onmessage({ data: { url: "/ror-locations.json", identifiers: ["https://ror.org/01qz5mb56", "https://ror.org/missing"] } });
    assert.deepEqual(JSON.parse(JSON.stringify(requests)), [{ url: "/ror-locations.json", options: { credentials: "omit", redirect: "error" } }]);
    assert.deepEqual(JSON.parse(JSON.stringify(messages[0].index)), { ...index, organizations: { "https://ror.org/01qz5mb56": index.organizations["https://ror.org/01qz5mb56"] } });
});

test("ROR worker rejects an oversized local index before parsing", async () => {
    const messages = [];
    const self = { postMessage: (message) => messages.push(message) };
    runInNewContext(source, {
        self,
        TextDecoder,
        fetch: async () => ({ ok: true, arrayBuffer: async () => ({ byteLength: 33554433 }) }),
    });
    await self.onmessage({ data: { url: "/ror-locations.json", identifiers: [] } });
    assert.match(messages[0].error, /Local registry too large/);
});
