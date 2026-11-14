import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { runInNewContext } from "node:vm";
import test from "node:test";
const context = { window: { acornWorkspace: { validPath: (path) => !path.split("/").some((part) => [".", "..", ""].includes(part)) && /\.(json|md|ya?ml|txt)$/.test(path) } }, URL, TextDecoder, Uint8Array, AbortController, setTimeout, clearTimeout, atob, Event, fetch };
runInNewContext(await readFile(new URL("remote.js", import.meta.url), "utf8"), context);
const sha = "a".repeat(40);
const response = (data, status = 200) => new Response(typeof data === "string" ? data : JSON.stringify(data), { status });
test("file URLs normalize repository links, retain queries and reject credentials or non-HTTPS schemes", () => {
    const parse = context.parseFileUrl;
    assert.equal(parse(" https://github.com/owner/repo/blob/main/data/file.json ").href, "https://raw.githubusercontent.com/owner/repo/main/data/file.json");
    assert.equal(parse("https://code.ornl.gov/group/repo/-/blob/main/file.yaml").pathname, "/group/repo/-/raw/main/file.yaml");
    assert.equal(parse("https://example.org/file.txt?download=1#part").href, "https://example.org/file.txt?download=1");
    for (const url of ["not a URL", "http://example.org/file.txt", "file:///tmp/data.json", "data:text/plain,hi", "https://user:secret@example.org/file.txt", "https://github.com/owner/repo"]) assert.throws(() => parse(url), url);
});
test("file loading bounds UTF-8 bytes and omits credentials and referrers", async () => {
    const signal = new AbortController().signal;
    const file = await context.loadRemoteFile("https://example.org/data?download=1", 4, signal, async (url, options) => {
        assert.equal(url, "https://example.org/data?download=1");
        assert.equal(options.credentials, "omit");
        assert.equal(options.referrerPolicy, "no-referrer");
        assert.equal(options.redirect, "follow");
        return new Response('"é"', { headers: { "Content-Type": "application/json" } });
    });
    assert.equal(file.path, "data.json");
    assert.equal(file.content, '"é"');
    assert.equal((await context.loadRemoteFile("https://example.org/empty.txt", 4, signal, async () => response(""))).content, "");
    await assert.rejects(context.loadRemoteFile("https://example.org/file.txt", 3, signal, async () => response('"é"')), /size limit/);
    await assert.rejects(context.loadRemoteFile("https://example.org/file.txt", 4, signal, async () => new Response("ok", { headers: { "content-length": "5" } })), /size limit/);
    await assert.rejects(context.loadRemoteFile("https://example.org/file.txt", 4, signal, async () => new Response(new Uint8Array([255]))), /UTF-8/);
    await assert.rejects(context.loadRemoteFile("https://example.org/file.txt", 4, signal, async () => response("missing", 404)), /HTTP 404/);
    const encoded = await context.loadRemoteFile("https://example.org/a%2fb.txt", 4, signal, async () => response("safe"));
    assert.equal(encoded.path, "download.txt");
});
test("file loader cancels and supersedes requests without returning stale content", async () => {
    const loader = context.createRemoteFileLoader(async (url, { signal }) => url.endsWith("slow.txt") ? new Promise((resolve, reject) => signal.addEventListener("abort", () => reject(signal.reason), { once: true })) : response("new"));
    const cancelled = loader.load("https://example.org/slow.txt", 10);
    loader.cancel();
    await assert.rejects(cancelled, /cancelled.*Current content is unchanged/);
    const previous = loader.load("https://example.org/slow.txt", 10);
    const rejected = assert.rejects(previous, /cancelled/);
    const next = await loader.load("https://example.org/new.txt", 10);
    await rejected;
    assert.equal(next.content, "new");
    await assert.rejects(loader.load("https://example.org/file.exe", 10, true), /JSON, YAML, Markdown or text/);
});
test("bucket history persists recent unique inputs, bounds suggestions and clears only its own key", () => {
    const values = new Map([["unrelated", "keep"]]);
    const storage = { getItem: (key) => values.get(key) ?? null, setItem: (key, value) => values.set(key, value), removeItem: (key) => values.delete(key) };
    const history = context.createBucketHistory(() => storage);
    for (let index = 0; index < 25; index += 1) history.remember(` owner/repo-${index} `, `release/${index}`);
    history.remember("owner/repo-24", "release/24");
    history.remember("", "  ");
    const reopened = context.createBucketHistory(() => storage).read();
    assert.equal(reopened.repositories.length, 20);
    assert.equal(reopened.refs.length, 20);
    assert.equal(reopened.repositories[0], "owner/repo-24");
    assert.equal(reopened.repositories.at(-1), "owner/repo-5");
    assert.equal(reopened.refs[0], "release/24");
    history.clear();
    assert.equal(history.read().repositories.length, 0);
    assert.equal(values.get("unrelated"), "keep");
});
test("bucket history tolerates unavailable storage and malformed saved values", () => {
    const unavailable = context.createBucketHistory(() => { throw new Error("disabled"); });
    assert.equal(unavailable.read().repositories.length, 0);
    assert.doesNotThrow(() => { unavailable.remember("owner/repo", "main"); unavailable.clear(); });
    const full = context.createBucketHistory(() => ({ getItem: () => null, setItem() { throw new Error("quota"); } }));
    assert.doesNotThrow(() => full.remember("owner/repo", "main"));
    for (const malformed of ["{", "null", "[]", '{"repositories":17,"refs":{}}']) {
        const history = context.createBucketHistory(() => ({ getItem: () => malformed }));
        assert.equal(history.read().repositories.length, 0);
        assert.equal(history.read().refs.length, 0);
    }
    const mixed = context.createBucketHistory(() => ({ getItem: () => JSON.stringify({ repositories: [null, {}, 4, " ", "owner/repo", "x".repeat(2049)], refs: [" main ", "main"] }) }));
    assert.deepEqual(Array.from(mixed.read().repositories), ["owner/repo"]);
    assert.deepEqual(Array.from(mixed.read().refs), ["main"]);
});
test("repository parser handles GitHub shortcuts, GitLab groups and slash refs", () => {
    const parse = context.parseRemoteBucket;
    assert.equal(parse("owner/repo").project, "owner/repo");
    const source = parse("https://code.ornl.gov/group/nested/bucket/-/tree/release/v1/data", "release/v1");
    assert.equal(source.project, "group/nested/bucket"); assert.equal(source.ref, "release/v1"); assert.equal(source.path, "data");
    assert.equal(parse("https://github.com/a/b/blob/main/rad.json").file, true);
    for (const url of ["http://github.com/a/b", "https://token@github.com/a/b", "https://github.com/a/b?token=x", "https://127.0.0.1/a/b", "https://github.com/a/b/issues", "https://github.com/a/b/tree"]) assert.throws(() => parse(url), url);
});
test("GitHub loads bounded supported files at the resolved commit without credentials", async () => {
    const seen = [];
    const result = await context.loadRemoteBucket(context.parseRemoteBucket("a/b"), new AbortController().signal, async (url, options) => {
        seen.push(url); assert.equal(options.credentials, "omit"); assert.equal(options.redirect, "error");
        if (url.endsWith("/a/b")) return response({ default_branch: "main" });
        if (url.endsWith("/commits/main")) return response({ sha });
        if (url.includes("/git/trees/")) return response({ tree: [{ type: "blob", path: "rad.json", sha, size: 2 }, { type: "blob", path: "image.png", sha }] });
        return response({ encoding: "base64", content: "e30=" });
    });
    assert.equal(result.files[0].content, "{}"); assert.equal(result.skipped, 1); assert.ok(seen[2].includes(sha));
});
test("GitLab paginates and loads raw files at a pinned commit", async () => {
    const seen = [];
    const result = await context.loadRemoteBucket(context.parseRemoteBucket("https://gitlab.com/a/b/-/tree/main/data"), new AbortController().signal, async (url) => {
        seen.push(url);
        if (url.endsWith("/projects/a%2Fb")) return response({ default_branch: "main" });
        if (url.includes("/commits/")) return response({ id: sha });
        if (url.includes("/tree?")) return response(url.includes("page=1&") ? Array.from({ length: 100 }, (_, i) => ({ type: "tree", path: `data/${i}` })) : [{ type: "blob", path: "data/rad.json" }]);
        return response('{}');
    });
    assert.equal(result.files.length, 1); assert.ok(seen.some((url) => url.includes("page=2"))); assert.ok(seen.at(-1).endsWith(`ref=${sha}`));
});
test("remote failures, truncated trees, oversized streams and cancellation reject atomically", async () => {
    await assert.rejects(context.readRemoteBytes(response("denied", 403), 10), /rate limit/);
    await assert.rejects(context.readRemoteBytes(response("12345"), 4), /size limit/);
    await assert.rejects(context.loadRemoteBucket(context.parseRemoteBucket("a/b"), new AbortController().signal, async (url) => response(url.endsWith("/a/b") ? { default_branch: "main" } : url.includes("/commits/") ? { sha } : { truncated: true, tree: [] })), /too large/);
    const controller = new AbortController(); controller.abort();
    await assert.rejects(context.loadRemoteBucket(context.parseRemoteBucket("a/b"), controller.signal, () => { throw Error("should not fetch"); }), /abort/i);
});
