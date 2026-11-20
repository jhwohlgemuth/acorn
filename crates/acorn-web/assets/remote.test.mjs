import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { runInNewContext } from "node:vm";
import test from "node:test";
const sessionValues = new Map();
const sessionStorage = { getItem: key => sessionValues.get(key) ?? null, setItem: (key, value) => sessionValues.set(key, value), removeItem: key => sessionValues.delete(key) };
const imageMime = (path) => /\.png$/i.test(path) ? "image/png" : /\.jpe?g$/i.test(path) ? "image/jpeg" : /\.webp$/i.test(path) ? "image/webp" : "";
const imageContent = (path, bytes) => {
    const mime = imageMime(path);
    if (!mime || !bytes.length || (mime === "image/png" && bytes[0] !== 137)) throw new Error("Image bytes do not match the file extension.");
    return `data:${mime};base64,${Buffer.from(bytes).toString("base64")}`;
};
const context = { window: { isSecureContext: true, sessionStorage, acornWorkspace: { validPath: (path) => !path.split("/").some((part) => [".", "..", ""].includes(part)) && /\.(json|md|ya?ml|txt|svg|png|jpe?g|webp)$/i.test(path), imageMime, imageContent } }, URL, TextDecoder, Uint8Array, AbortController, setTimeout, clearTimeout, atob, Event, fetch };
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
test("workspace image URLs retain bytes while text-tool URLs reject binary input", async () => {
    const png = Buffer.from("iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+/RZkAAAAASUVORK5CYII=", "base64");
    const loader = context.createRemoteFileLoader(async () => new Response(png));
    const loaded = await loader.load("https://example.org/cover.png", 262144, true);
    assert.equal(loaded.content, `data:image/png;base64,${png.toString("base64")}`);
    await assert.rejects(loader.load("https://example.org/cover.png", 262144), /UTF-8/);
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
    await assert.rejects(loader.load("https://example.org/file.exe", 10, true), /supported image/);
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
    assert.equal(history.clear(), true);
    assert.equal(history.read().repositories.length, 0);
    assert.equal(values.get("unrelated"), "keep");
});
test("bucket history tolerates unavailable storage and malformed saved values", () => {
    const unavailable = context.createBucketHistory(() => { throw new Error("disabled"); });
    assert.equal(unavailable.read().repositories.length, 0);
    assert.doesNotThrow(() => { unavailable.remember("owner/repo", "main"); assert.equal(unavailable.clear(), false); });
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
        seen.push(url); assert.equal(options.credentials, "omit"); assert.equal(options.redirect, "follow");
        if (url.endsWith("/a/b")) return response({ default_branch: "main" });
        if (url.endsWith("/commits/main")) return response({ sha });
        if (url.includes("/git/trees/")) return response({ tree: [{ type: "blob", path: "rad.json", sha, size: 2 }, { type: "blob", path: "ignored.bin", sha }] });
        return response({ encoding: "base64", content: "e30=" });
    });
    assert.equal(result.files[0].content, "{}"); assert.equal(result.skipped, 1); assert.ok(seen[2].includes(sha));
    assert.equal(result.reasons.unsupported, 1);
});
test("pinned GitHub and GitLab imports retain image bytes for the catalog", async () => {
    const png = Buffer.from("iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+/RZkAAAAASUVORK5CYII=", "base64");
    for (const kind of ["github", "gitlab"]) {
        const source = context.parseRemoteBucket(kind === "github" ? "a/b" : "https://gitlab.com/a/b");
        const result = await context.loadRemoteBucket(source, new AbortController().signal, async (url) => {
            if (url.endsWith("/a/b") || url.endsWith("/projects/a%2Fb")) return response({ default_branch: "main" });
            if (url.includes("/commits/")) return response(kind === "github" ? { sha } : { id: sha });
            if (url.includes("/git/trees/")) return response({ tree: [{ type: "blob", path: "project/cover.png", sha }] });
            if (url.includes("/repository/tree?")) return response([{ type: "blob", path: "project/cover.png" }]);
            return kind === "github" ? response({ encoding: "base64", content: png.toString("base64") }) : new Response(png);
        });
        assert.equal(result.files[0].path, "project/cover.png");
        assert.equal(result.files[0].content, `data:image/png;base64,${png.toString("base64")}`);
        assert.equal(result.source.revision, sha);
    }
});
test("GitLab requests LFS image content and skips invalid image bytes", async () => {
    const seen = [];
    const source = context.parseRemoteBucket("https://code.ornl.gov/group/repo");
    const result = await context.loadRemoteBucket(source, new AbortController().signal, async (url) => {
        seen.push(url);
        if (url.endsWith("/projects/group%2Frepo")) return response({ id: 42, default_branch: "main" });
        if (url.includes("/commits/")) return response({ id: sha });
        if (url.includes("/tree?")) return response([{ type: "blob", path: "bad.png" }, { type: "blob", path: "rad.json" }]);
        return response(url.includes("bad.png") ? "version https://git-lfs.github.com/spec/v1" : "{}");
    });
    assert.deepEqual(Array.from(result.files, (file) => file.path), ["rad.json"]);
    assert.equal(result.skipped, 1);
    assert.equal(result.unavailable, 0);
    assert.equal(result.reasons.invalidContent, 1);
    assert.ok(seen.filter((url) => url.includes("/repository/files/")).every((url) => url.endsWith(`ref=${sha}&lfs=true`)));
});
test("repository token is header-only and redirects can be followed", async () => {
    const source = context.parseRemoteBucket("a/b");
    const seen = [];
    const load = (token) => context.loadRemoteBucket(source, new AbortController().signal, async (url, options) => {
        seen.push({ url, options });
        if (url.endsWith("/a/b")) return response({ default_branch: "main" });
        if (url.endsWith("/commits/main")) return response({ sha });
        if (url.includes("/git/trees/")) return response({ tree: [{ type: "blob", path: "rad.json", sha }] });
        return response({ encoding: "base64", content: "e30=" });
    }, token);
    await load("secret-value");
    assert.ok(seen.every(({ url, options }) => !url.includes("secret-value") && options.headers.Authorization === "Bearer secret-value" && options.redirect === "follow"));
    seen.length = 0;
    await load("");
    assert.ok(seen.every(({ options }) => !Object.hasOwn(options.headers, "Authorization")));
});
test("repository credential can be saved before a URL and used across repository hosts", async () => {
    const seen = [];
    context.window.acornWorkspace.cancel = () => {};
    context.fetch = async (url, options) => {
        seen.push({ url, options });
        if (url.endsWith("/a/b") || url.endsWith("/a/c") || url.endsWith("/projects/a%2Fb") || url.endsWith("/projects/a%2Fc")) return response({ default_branch: "main" });
        if (url.includes("/commits/")) return response(url.includes("api.github.com") ? { sha } : { id: sha });
        if (url.includes("/git/trees/")) return response({ tree: [{ type: "blob", path: "rad.json", sha }] });
        if (url.includes("/repository/tree?")) return response([{ type: "blob", path: "rad.json" }]);
        return url.includes("api.github.com") ? response({ encoding: "base64", content: "e30=" }) : response("{}");
    };
    const loader = context.createRemoteLoader();
    assert.equal(loader.setCredential("secret-value"), "all repository URLs");
    assert.equal(context.createRemoteLoader().credentialStatus(), "all repository URLs", "same-tab reload reads session storage");
    await loader.load("a/b", "");
    assert.ok(seen.every(({ options }) => options.headers.Authorization === "Bearer secret-value"));
    seen.length = 0;
    await loader.load("a/c", "");
    assert.ok(seen.every(({ options }) => options.headers.Authorization === "Bearer secret-value"), "a second GitHub repository shares its provider host token");
    seen.length = 0;
    await loader.load("https://gitlab.com/a/b", "");
    assert.ok(seen.every(({ url, options }) => !url.includes("secret-value") && options.headers.Authorization === "Bearer secret-value" && options.redirect === "follow"));
    seen.length = 0;
    await loader.load("https://gitlab.com/a/c", "");
    assert.ok(seen.every(({ options }) => options.headers.Authorization === "Bearer secret-value"));
    seen.length = 0;
    await loader.load("https://code.ornl.gov/a/b", "");
    assert.ok(seen.every(({ options }) => options.headers.Authorization === "Bearer secret-value"), "the app token follows the selected repository URL");
    loader.clearCredential();
    assert.equal(loader.credentialStatus(), "");
    assert.equal(sessionValues.has("acorn.repository-credential.v1"), false);
    assert.equal(loader.history.read().repositories.includes("secret-value"), false);
});
test("an existing project-bound tab token stays project-bound until explicitly replaced", async () => {
    const key = "acorn.repository-credential.v1";
    sessionStorage.setItem(key, JSON.stringify({ kind: "gitlab", host: "code.ornl.gov", project: "group/first", token: "legacy-token" }));
    context.window.acornWorkspace.cancel = () => {};
    const seen = [];
    context.fetch = async (url, options) => { seen.push({ url, options }); return response({ message: "401 Unauthorized" }, 401); };
    const loader = context.createRemoteLoader();
    assert.match(loader.credentialStatus(), /code\.ornl\.gov\/group\/first \(repository only/);
    await assert.rejects(loader.load("https://code.ornl.gov/group/first", ""), /rejected the configured token/);
    assert.equal(seen[0].options.headers.Authorization, "Bearer legacy-token");
    seen.length = 0;
    await assert.rejects(loader.load("https://code.ornl.gov/group/second", ""), /Save a repository token/);
    assert.equal(Object.hasOwn(seen[0].options.headers, "Authorization"), false);
    assert.equal(loader.setCredential("legacy-token"), "all repository URLs");
    assert.equal(loader.credentialStatus(), "all repository URLs");
    seen.length = 0;
    await assert.rejects(loader.load("https://code.ornl.gov/group/second", ""), /rejected the configured token/);
    assert.equal(seen[0].options.headers.Authorization, "Bearer legacy-token");
    loader.clearCredential();
});
test("repository tokens require secure session storage", () => {
    const credentials = context.createRepositoryCredentials(() => sessionStorage, () => false);
    assert.throws(() => credentials.set("secret"), /secure browser context/);
    assert.equal(credentials.read(), null);
    const unavailable = context.createRepositoryCredentials(() => { throw new Error("denied"); }, () => true);
    assert.equal(unavailable.read(), null);
    assert.throws(() => unavailable.set("secret"), /Session storage is unavailable/);
    sessionStorage.setItem("acorn.repository-credential.v1", "{bad json");
    assert.equal(context.createRepositoryCredentials(() => sessionStorage, () => true).read(), null);
    sessionStorage.removeItem("acorn.repository-credential.v1");
});
test("GitLab paginates and loads raw files at a pinned commit", async () => {
    const seen = [];
    const cursor = "eyJmaWxlX25hbWUiOiIuZHZjIiwidHJlZV9vaWQiOiJmNzY1MzYxZDUwZDJhMWE4OTFkOGFiZGFhOGRjYjdjMzBkZjRlOTQxIn0=";
    const result = await context.loadRemoteBucket(context.parseRemoteBucket("https://gitlab.com/a/b/-/tree/main/data"), new AbortController().signal, async (url) => {
        seen.push(url);
        if (url.endsWith("/projects/a%2Fb")) return response({ default_branch: "main" });
        if (url.includes("/commits/")) return response({ id: sha });
        if (url.includes("/tree?")) return url.includes("page_token=")
            ? response([{ type: "blob", path: "data/rad.json" }])
            : new Response(JSON.stringify(Array.from({ length: 100 }, (_, i) => ({ id: sha, type: "tree", path: `data/${i}` }))), { headers: { Link: `<https://gitlab.com/api/v4/projects/a%2Fb/repository/tree?page_token=${encodeURIComponent(cursor)}&pagination=keyset>; rel="next"` } });
        return response('{}');
    });
    assert.equal(result.files.length, 1); assert.ok(seen.some((url) => url.includes(`page_token=${encodeURIComponent(cursor)}`))); assert.ok(seen.at(-1).endsWith(`ref=${sha}&lfs=true`));
});
test("GitLab uses the resolved numeric project ID for repository requests", async () => {
    const seen = [];
    const source = context.parseRemoteBucket("https://code.ornl.gov/research-enablement/buckets/nssd");
    const result = await context.loadRemoteBucket(source, new AbortController().signal, async (url) => {
        seen.push(url);
        if (url.endsWith("/projects/research-enablement%2Fbuckets%2Fnssd")) return response({ id: 42, default_branch: "main" });
        if (url.includes("/commits/")) return response({ id: sha });
        if (url.includes("/tree?")) return response([{ type: "blob", path: "rad.json" }]);
        return response("{}");
    });
    assert.equal(result.files[0].content, "{}");
    assert.ok(seen.slice(1).every((url) => url.startsWith("https://code.ornl.gov/api/v4/projects/42/")));
});
test("repository requests accept same-origin redirects and reject another host", async () => {
    const source = context.parseRemoteBucket("https://code.ornl.gov/group/repo");
    const redirected = (url, data) => {
        const result = response(data);
        Object.defineProperty(result, "url", { value: url });
        return result;
    };
    const fetcher = async (url, options) => {
        assert.equal(options.redirect, "follow");
        if (url.endsWith("/projects/group%2Frepo")) return redirected("https://code.ornl.gov/api/v4/projects/42", { id: 42, default_branch: "main" });
        if (url.includes("/commits/")) return response({ id: sha });
        if (url.includes("/tree?")) return response([{ type: "blob", path: "rad.json" }]);
        return response("{}");
    };
    const result = await context.loadRemoteBucket(source, new AbortController().signal, fetcher, "private-demo-token");
    assert.equal(result.files[0].content, "{}");
    await assert.rejects(context.loadRemoteBucket(source, new AbortController().signal, async () => redirected("https://other.example.org/api/v4/projects/42", { id: 42, default_branch: "main" }), "private-demo-token"), /redirected to another host/);
});
test("GitLab accepts only same-host tree cursors from the next link", () => {
    const origin = "https://code.ornl.gov";
    const next = `<${origin}/api/v4/projects/23285/repository/tree?page_token=opaque%3D>; rel="next"`;
    assert.equal(context.nextGitLabPageToken(next, origin), "opaque=");
    assert.equal(context.nextGitLabPageToken(null, origin), "");
    assert.throws(() => context.nextGitLabPageToken('<https://other.example.org/repository/tree?page_token=opaque>; rel="next"', origin), /invalid listing cursor/);
    assert.throws(() => context.nextGitLabPageToken(`<${origin}/api/v4/projects/23285/repository/tree>; rel="next"`, origin), /invalid listing cursor/);
});
test("GitLab identifies the failing listing page", async () => {
    const source = context.parseRemoteBucket("https://code.ornl.gov/research-enablement/buckets/nssd");
    const next = `<https://code.ornl.gov/api/v4/projects/23285/repository/tree?page_token=opaque>; rel="next"`;
    await assert.rejects(context.loadRemoteBucket(source, new AbortController().signal, async (url) => {
        if (url.endsWith("/projects/research-enablement%2Fbuckets%2Fnssd")) return response({ default_branch: "main" });
        if (url.includes("/commits/")) return response({ id: sha });
        if (url.includes("page_token=")) throw new TypeError("Failed to fetch");
        return new Response(JSON.stringify([{ type: "tree", path: "folder" }]), { headers: { Link: next } });
    }), /file listing.*Listing page 2/);
});
test("GitLab loads files above the old limit and skips oversized files", async () => {
    const result = await context.loadRemoteBucket(context.parseRemoteBucket("https://code.ornl.gov/research-enablement/buckets/nssd"), new AbortController().signal, async (url) => {
        if (url.endsWith("/projects/research-enablement%2Fbuckets%2Fnssd")) return response({ default_branch: "main" });
        if (url.includes("/commits/")) return response({ id: sha });
        if (url.includes("/tree?")) return response([
            { type: "blob", path: "data/large.txt" },
            { type: "blob", path: "data/small.json" },
            { type: "blob", path: "data/too-large.txt" },
        ]);
        if (url.includes("too-large.txt")) return new Response("too large", { headers: { "content-length": String(2 * 1024 * 1024 + 1) } });
        return response(url.includes("large.txt") ? "x".repeat(300 * 1024) : "{}");
    });
    assert.deepEqual(Array.from(result.files, (file) => file.path), ["data/large.txt", "data/small.json"]);
    assert.equal(result.files[0].content.length, 300 * 1024);
    assert.equal(result.skipped, 1);
    assert.equal(result.reasons.fileLimit, 1);
});
test("GitLab keeps available files when one raw request fails", async () => {
    const source = context.parseRemoteBucket("https://code.ornl.gov/research-enablement/buckets/nssd");
    const result = await context.loadRemoteBucket(source, new AbortController().signal, async (url) => {
        if (url.endsWith("/projects/research-enablement%2Fbuckets%2Fnssd")) return response({ default_branch: "main" });
        if (url.includes("/commits/")) return response({ id: sha });
        if (url.includes("/tree?")) return response([{ type: "blob", path: "bad.json" }, { type: "blob", path: "good.json" }]);
        if (url.includes("bad.json")) throw new TypeError("Failed to fetch");
        return response("{}");
    }, "private-demo-token");
    assert.deepEqual(Array.from(result.files, (file) => file.path), ["good.json"]);
    assert.equal(result.skipped, 1);
    assert.equal(result.unavailable, 1);
    assert.equal(result.reasons.unavailable, 1);
});
test("GitLab retries a failed path download through its pinned blob", async () => {
    const seen = [];
    const source = context.parseRemoteBucket("https://code.ornl.gov/research-enablement/buckets/nssd");
    const result = await context.loadRemoteBucket(source, new AbortController().signal, async (url) => {
        seen.push(url);
        if (url.endsWith("/projects/research-enablement%2Fbuckets%2Fnssd")) return response({ default_branch: "main" });
        if (url.includes("/commits/")) return response({ id: sha });
        if (url.includes("/tree?")) return response([{ id: sha, type: "blob", path: "data/rad.json" }]);
        if (url.includes("/repository/files/")) throw new TypeError("Failed to fetch");
        return response("{}");
    }, "private-demo-token");
    assert.equal(result.files[0].content, "{}");
    assert.equal(result.unavailable, 0);
    assert.ok(seen.some((url) => url.endsWith(`/repository/blobs/${sha}/raw`)));
});
test("GitLab identifies the request stage when network access fails", async () => {
    const source = context.parseRemoteBucket("https://code.ornl.gov/research-enablement/buckets/nssd");
    await assert.rejects(context.loadRemoteBucket(source, new AbortController().signal, async () => { throw new TypeError("Failed to fetch"); }, "private-demo-token"), /project details.*network or CORS.*saved token was sent/i);
});
test("private GitLab load keeps its token on the selected host and redacts failures", async () => {
    context.window.acornWorkspace.cancel = () => {};
    const calls = [];
    context.fetch = async (url, options) => {
        calls.push({ url, options });
        if (url.endsWith("/projects/a%2Fb")) return response({ default_branch: "main" });
        if (url.includes("/commits/")) return response({ id: sha });
        if (url.includes("/repository/tree?")) return response([{ type: "blob", path: "rad.json" }]);
        return response("{}");
    };
    const loader = context.createRemoteLoader();
    loader.setCredential("private-demo-token");
    const loaded = await loader.load("https://gitlab.com/a/b", "");
    assert.equal(loaded.source.revision, sha);
    assert.equal(loaded.files[0].path, "rad.json");
    assert.ok(calls.every(({ url, options }) => url.startsWith("https://gitlab.com/api/v4/projects/a%2Fb") && options.headers.Authorization === "Bearer private-demo-token" && options.redirect === "follow"));
    context.fetch = async () => { throw new Error("private-demo-token rejected"); };
    await assert.rejects(loader.load("https://gitlab.com/a/b", ""), (error) => !error.message.includes("private-demo-token") && error.message.includes("[redacted]"));
    loader.clearCredential();
    assert.equal(sessionValues.has("acorn.repository-credential.v1"), false);
});
test("nested GitLab project 401 distinguishes a rejected token from an unmatched host", async () => {
    const project = "https://code.ornl.gov/research-enablement/buckets/nssd";
    const seen = [];
    context.window.acornWorkspace.cancel = () => {};
    context.fetch = async (url, options) => { seen.push({ url, options }); return response({ message: "401 Unauthorized" }, 401); };
    const loader = context.createRemoteLoader();
    assert.equal(loader.setCredential("  legacy-pat  "), "all repository URLs");
    await assert.rejects(loader.load(project, ""), /configured token \(401\) while reading project details/);
    assert.equal(seen[0].url, "https://code.ornl.gov/api/v4/projects/research-enablement%2Fbuckets%2Fnssd");
    assert.equal(seen[0].options.headers.Authorization, "Bearer legacy-pat");
    seen.length = 0;
    await assert.rejects(loader.load("https://other.example.org/research-enablement/buckets/other", ""), /rejected the configured token \(401\)/);
    assert.equal(seen[0].options.headers.Authorization, "Bearer legacy-pat");
    loader.clearCredential();
});
test("remote failures, truncated trees, oversized streams and cancellation reject atomically", async () => {
    await assert.rejects(context.readRemoteBytes(response("denied", 403), 10), /rate limit/);
    await assert.rejects(context.readRemoteBytes(response("missing", 404), 10), /save a repository token/);
    await assert.rejects(context.readRemoteBytes(response("12345"), 4), /size limit/);
    await assert.rejects(context.loadRemoteBucket(context.parseRemoteBucket("a/b"), new AbortController().signal, async (url) => response(url.endsWith("/a/b") ? { default_branch: "main" } : url.includes("/commits/") ? { sha } : { truncated: true, tree: [] })), /too large/);
    const controller = new AbortController(); controller.abort();
    await assert.rejects(context.loadRemoteBucket(context.parseRemoteBucket("a/b"), controller.signal, () => { throw Error("should not fetch"); }), /abort/i);
});
