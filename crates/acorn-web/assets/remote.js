const remoteLimits = { entries: 10000, file: 2 * 1024 * 1024, files: 500, listing: 10 * 1024 * 1024, total: 32 * 1024 * 1024 };
class RemoteSizeError extends Error {}
class RemoteTransportError extends Error {}
function createBucketHistory(storage = () => window.localStorage) {
    const key = "acorn.bucket-history.v1";
    const recent = (values) => [...new Set((Array.isArray(values) ? values : []).filter((value) => typeof value === "string" && value.length <= 2048).map((value) => value.trim()).filter(Boolean))].slice(0, 20);
    const read = () => {
        try {
            const saved = JSON.parse(storage().getItem(key));
            return { repositories: recent(saved?.repositories), refs: recent(saved?.refs) };
        } catch { return { repositories: [], refs: [] }; }
    };
    return {
        clear() { try { storage().removeItem(key); return true; } catch { return false; } },
        read,
        remember(input, branch) {
            const saved = read();
            const next = { repositories: recent([input, ...saved.repositories]), refs: recent([branch, ...saved.refs]) };
            try { storage().setItem(key, JSON.stringify(next)); } catch { /* History is optional when storage is unavailable or full. */ }
        },
    };
}
function createRemoteFileLoader(fetcher = fetch) {
    let active;
    return {
        cancel() { active?.abort(); active = undefined; },
        async load(input, maximum, workspaceFile = false) {
            window.acornWorkspace?.cancel?.();
            active?.abort();
            const controller = new AbortController();
            active = controller;
            const timeout = setTimeout(() => controller.abort(), 30000);
            try {
                const file = await loadRemoteFile(input, maximum, controller.signal, fetcher, workspaceFile);
                controller.signal.throwIfAborted();
                if (workspaceFile && !window.acornWorkspace.validPath(file.path)) throw new Error("Choose a JSON, YAML, Markdown, text or supported image file for this workspace.");
                return file;
            } catch (error) {
                const message = controller.signal.aborted ? "File download cancelled or timed out." : error instanceof TypeError ? "Could not fetch this file. Check the URL and network connection; the server must allow browser access (CORS). You can download the file and choose it locally instead." : error.message;
                throw new Error(`${message} Current content is unchanged.`);
            } finally { clearTimeout(timeout); if (active === controller) active = undefined; }
        },
    };
}
function createRemoteLoader() {
    let active;
    const credentials = createRepositoryCredentials();
    const file = createRemoteFileLoader();
    const history = createBucketHistory();
    return {
        cancel() { active?.abort(); active = undefined; file.cancel(); },
        clearData() { const historyCleared = history.clear(); const credentialCleared = credentials.clear(); return historyCleared && credentialCleared; },
        file,
        history,
        credentialStatus() {
            const saved = credentials.read();
            if (!saved) return "";
            if (saved.scope === "app") return "all repository URLs";
            return saved.scope === "host" ? `${saved.host} (save again for all URLs)` : `${saved.host}/${saved.project} (repository only; save again for all URLs)`;
        },
        clearCredential() { return credentials.clear(); },
        setCredential(token) {
            const cleaned = token.trim();
            if (!cleaned || cleaned.length > 4096 || /[\r\n]/.test(cleaned)) throw new Error("Enter a valid token.");
            credentials.set(cleaned);
            return "all repository URLs";
        },
        async load(input, branch) {
            window.acornWorkspace.cancel();
            const controller = new AbortController();
            active = controller;
            const timeout = setTimeout(() => controller.abort(), 120000);
            let token = "";
            try {
                const source = parseRemoteBucket(input, branch);
                history.remember(input, branch);
                const credential = credentials.read();
                token = credential && (credential.scope === "app" || credential.host === new URL(source.origin).host && credential.kind === source.kind && (credential.scope === "host" || credential.project === source.project)) ? credential.token : "";
                return await loadRemoteBucket(source, controller.signal, fetch, token);
            }
            catch (error) {
                const message = controller.signal.aborted ? "Remote load cancelled or timed out. Your previous session is unchanged." : error instanceof TypeError ? "Could not load this repository. Check the URL, network connection and the server’s CORS settings." : error instanceof SyntaxError ? "The repository returned invalid data." : error.message;
                throw new Error(token ? message.replaceAll(token, "[redacted]") : message);
            }
            finally { clearTimeout(timeout); if (active === controller) active = undefined; }
        },
    };
}
function createRepositoryCredentials(storage = () => window.sessionStorage, secure = () => window.isSecureContext) {
    const key = "acorn.repository-credential.v1";
    const read = () => {
        if (!secure()) return null;
        try {
            const value = JSON.parse(storage().getItem(key));
            const token = typeof value?.token === "string" && value.token.length > 0 && value.token.length <= 4096 && !/[\r\n]/.test(value.token);
            const legacy = ["github", "gitlab"].includes(value?.kind) && typeof value?.host === "string";
            const scope = value?.scope === "app" ? "app" : legacy && value?.scope === "host" ? "host" : legacy && value?.scope === undefined && typeof value?.project === "string" ? "project" : null;
            return token && scope ? { ...value, scope } : null;
        } catch { return null; }
    };
    return {
        read,
        clear() { try { storage().removeItem(key); return true; } catch { return false; } },
        set(token) {
            if (!secure()) throw new Error("Repository tokens require a secure browser context.");
            const credential = { scope: "app", token };
            try { storage().setItem(key, JSON.stringify(credential)); } catch { throw new Error("Session storage is unavailable. Repository token was not saved."); }
            return credential;
        },
    };
}
async function loadRemoteBucket(source, signal, fetcher = fetch, token = "") {
    let base = source.kind === "github" ? `${source.origin}/repos/${source.project.split("/").map(encodeURIComponent).join("/")}` : `${source.origin}/api/v4/projects/${encodeURIComponent(source.project)}`;
    const request = async (path, maximum = remoteLimits.listing) => {
        signal.throwIfAborted();
        const headers = { Accept: "application/json" };
        if (token) headers.Authorization = `Bearer ${token}`;
        const step = !path ? "project details" : path.includes("/commits/") ? "the commit" : path.includes("/trees/") || path.startsWith("/repository/tree?") ? "the file listing" : "a repository file";
        try {
            const response = await fetcher(base + path, { signal, credentials: "omit", redirect: "follow", referrerPolicy: "no-referrer", headers });
            if (response.url && new URL(response.url).origin !== source.origin) throw new Error(`Repository request for ${step} redirected to another host. Use a repository URL on its final HTTPS host.`);
            if (source.kind === "gitlab" && response.status === 401) {
                throw new Error(token
                    ? `GitLab rejected the configured token (401) while reading ${step}. Check that the token is active, has read_api, and its owner can access this project.`
                    : `GitLab requires authentication (401) while reading ${step}. Save a repository token in this tab.`);
            }
            return { bytes: await readRemoteBytes(response, maximum), link: response.headers.get("link") };
        } catch (error) {
            if (error?.name === "TypeError") throw new RemoteTransportError(`Could not load ${step}: the browser blocked or lost the request (network or CORS). ${token ? "A saved token was sent." : "No repository token was sent."}`);
            throw error;
        }
    };
    const json = async (path) => JSON.parse(new TextDecoder("utf-8", { fatal: true }).decode((await request(path)).bytes));
    const repository = await json("");
    if (source.kind === "gitlab" && Number.isSafeInteger(repository.id) && repository.id > 0) base = `${source.origin}/api/v4/projects/${repository.id}`;
    const ref = source.ref || repository.default_branch;
    if (!ref) throw new Error("This repository has no default branch.");
    const commit = await json(`${source.kind === "github" ? "/commits/" : "/repository/commits/"}${encodeURIComponent(ref)}`);
    const revision = source.kind === "github" ? commit.sha : commit.id;
    if (!/^[a-f0-9]{40,64}$/i.test(revision || "")) throw new Error("Repository returned an invalid revision.");
    let entries = [];
    if (source.kind === "github") {
        const tree = await json(`/git/trees/${revision}?recursive=1`);
        if (tree.truncated || !Array.isArray(tree.tree) || tree.tree.length > remoteLimits.entries) throw new Error("Repository listing is too large. Load a smaller repository or a local folder.");
        entries = tree.tree;
    } else if (source.file) {
        entries = [{ path: source.path, type: "blob" }];
    } else {
        let pageToken = "";
        for (let page = 1; page <= 100; page += 1) {
            const cursor = pageToken ? `&page_token=${encodeURIComponent(pageToken)}` : "";
            let listing;
            try { listing = await request(`/repository/tree?recursive=true&pagination=keyset&per_page=100&ref=${revision}&path=${encodeURIComponent(source.path)}${cursor}`); }
            catch (error) {
                if (error instanceof RemoteTransportError) throw new RemoteTransportError(`${error.message} Listing page ${page}.`);
                throw error;
            }
            const batch = JSON.parse(new TextDecoder("utf-8", { fatal: true }).decode(listing.bytes));
            if (!Array.isArray(batch)) throw new Error("Invalid repository listing.");
            entries.push(...batch);
            if (entries.length > remoteLimits.entries) throw new Error("Folder contains too many entries. Choose a smaller folder.");
            const next = nextGitLabPageToken(listing.link, source.origin);
            if (!next) break;
            if (page === 100 || next === pageToken) throw new Error("Repository listing exceeds the page limit or returned a repeated cursor.");
            pageToken = next;
        }
    }
    const inside = (path) => !source.path || path === source.path || (!source.file && path.startsWith(source.path + "/"));
    const candidates = entries.filter((entry) => entry.type === "blob" && entry.mode !== "120000" && typeof entry.path === "string" && inside(entry.path));
    const reasons = { unsupported: 0, fileLimit: 0, invalidContent: 0, unavailable: 0, totalLimit: 0 };
    const accepted = candidates.filter((entry) => {
        if (!window.acornWorkspace.validPath(entry.path)) { reasons.unsupported += 1; return false; }
        if (entry.size !== undefined && entry.size > remoteLimits.file) { reasons.fileLimit += 1; return false; }
        return true;
    }).sort((a, b) => a.path.localeCompare(b.path));
    if (accepted.length > remoteLimits.files) throw new Error("More than 500 supported files. Choose a smaller folder.");
    const files = [];
    let skipped = candidates.length - accepted.length;
    let unavailable = 0;
    let failedRequest;
    let total = 0;
    for (const entry of accepted) {
        signal.throwIfAborted();
        try {
            let bytes;
            if (source.kind === "github") {
                if (!/^[a-f0-9]{40,64}$/i.test(entry.sha || "")) throw new Error("Invalid repository file identity.");
                const blob = JSON.parse(new TextDecoder().decode((await request(`/git/blobs/${entry.sha}`, remoteLimits.file * 2)).bytes));
                if (blob.encoding !== "base64" || typeof blob.content !== "string") throw new Error("Unsupported repository file encoding.");
                bytes = Uint8Array.from(atob(blob.content.replace(/\s/g, "")), (char) => char.charCodeAt(0));
            } else {
                try {
                    bytes = (await request(`/repository/files/${encodeURIComponent(entry.path)}/raw?ref=${revision}&lfs=true`, remoteLimits.file)).bytes;
                } catch (error) {
                    if (error instanceof RemoteTransportError && /^[a-f0-9]{40,64}$/i.test(entry.id || "")) bytes = (await request(`/repository/blobs/${entry.id}/raw`, remoteLimits.file)).bytes;
                    else throw error;
                }
            }
            if (bytes.byteLength > remoteLimits.file) throw new RemoteSizeError("Repository file exceeds the size limit.");
            const image = window.acornWorkspace.imageMime(entry.path);
            let content;
            try { content = image ? window.acornWorkspace.imageContent(entry.path, bytes) : new TextDecoder("utf-8", { fatal: true }).decode(bytes); }
            catch { skipped += 1; reasons.invalidContent += 1; continue; }
            const storedBytes = image ? content.length : bytes.byteLength;
            if (total + storedBytes > remoteLimits.total) throw new RemoteSizeError("Repository exceeds the stored total size limit.");
            total += storedBytes;
            files.push({ path: entry.path, content });
        } catch (error) {
            if (error instanceof RemoteSizeError || error instanceof RemoteTransportError) {
                skipped += 1;
                if (error instanceof RemoteTransportError) { unavailable += 1; reasons.unavailable += 1; failedRequest ??= error; }
                else if (error.message.includes("stored total")) reasons.totalLimit += 1;
                else reasons.fileLimit += 1;
            }
            else throw error;
        }
    }
    signal.throwIfAborted();
    if (!files.length) throw failedRequest || new Error("No supported research files found at this location.");
    return { files, skipped, unavailable, reasons, source: { kind: source.kind, host: new URL(source.origin).host, project: source.project, path: source.path, revision } };
}
async function loadRemoteFile(input, maximum, signal, fetcher = fetch, workspaceFile = false) {
    const url = parseFileUrl(input);
    if (!Number.isSafeInteger(maximum) || maximum < 1 || maximum > remoteLimits.file) throw new Error("Invalid file size limit.");
    signal.throwIfAborted();
    const response = await fetcher(url.href, { signal, credentials: "omit", redirect: "follow", referrerPolicy: "no-referrer" });
    if (!response.ok) throw new Error(`Could not load this file (HTTP ${response.status}). Check the URL and access permissions.`);
    if (response.url) parseFileUrl(response.url);
    const bytes = await readRemoteBytes(response, maximum);
    signal.throwIfAborted();
    const name = decodeURIComponent(url.pathname.split("/").pop() || "download.txt");
    // oxlint-disable-next-line no-control-regex -- Remote file names must reject control characters.
    const path = !name || name === "." || name === ".." || /[\\/\x00-\x1f\x7f]/.test(name) ? "download.txt" : name;
    const extension = /json/i.test(response.headers.get("content-type") || "") ? ".json" : /yaml/i.test(response.headers.get("content-type") || "") ? ".yaml" : ".txt";
    const resolved = path.includes(".") ? path : path + extension;
    let content;
    if (workspaceFile && window.acornWorkspace.imageMime(resolved)) content = window.acornWorkspace.imageContent(resolved, bytes);
    else {
        try { content = new TextDecoder("utf-8", { fatal: true }).decode(bytes); }
        catch { throw new Error("Choose a UTF-8 text file."); }
    }
    return { path: resolved, content };
}
function parseFileUrl(input) {
    let url;
    try { url = new URL(input.trim()); } catch { throw new Error("Enter a complete HTTPS file URL."); }
    if (url.protocol !== "https:" || url.username || url.password) throw new Error("Use an HTTPS file URL without embedded credentials.");
    url.hash = "";
    if (["github.com", "www.github.com"].includes(url.hostname)) {
        const parts = url.pathname.split("/");
        if (parts[3] !== "blob" || parts.length < 6) throw new Error("Use a GitHub file link containing /blob/ or a raw file URL.");
        url.hostname = "raw.githubusercontent.com";
        parts.splice(3, 1);
        url.pathname = parts.join("/");
    } else if (url.pathname.includes("/-/blob/")) {
        url.pathname = url.pathname.replace("/-/blob/", "/-/raw/");
    }
    return url;
}
function parseRemoteBucket(input, branch = "") {
    const value = input.trim();
    const url = new URL(/^[\w.-]+\/[\w.-]+$/.test(value) ? `https://github.com/${value}` : value);
    if (url.protocol !== "https:" || url.username || url.password || url.search || url.hash || url.port || !url.hostname.includes(".") || /^(localhost|127\.|10\.|192\.168\.|169\.254\.|172\.(1[6-9]|2\d|3[01])\.)/.test(url.hostname) || url.hostname.startsWith("[")) throw new Error("Use a public HTTPS GitHub or GitLab repository URL without credentials or query parameters.");
    const parts = url.pathname.split("/").filter(Boolean).map(decodeURIComponent);
    // oxlint-disable-next-line no-control-regex -- Remote repository paths must reject control characters.
    if (parts.some((part) => !part || part === "." || part === ".." || /[\\\x00-\x1f\x7f]/.test(part))) throw new Error("Invalid repository path.");
    const github = url.hostname === "github.com" || url.hostname === "www.github.com";
    const marker = github ? 2 : parts.indexOf("-");
    const project = (marker < 0 ? parts : parts.slice(0, marker)).join("/").replace(/\.git$/, "");
    if ((github && parts.length < 2) || (!github && project.split("/").length < 2)) throw new Error("Include the repository owner or GitLab group and project.");
    const route = marker < 0 ? [] : parts.slice(marker);
    if (!github && route[0] === "-") route.shift();
    if (route.length && !["tree", "blob", "raw"].includes(route[0])) throw new Error("Use a repository, folder (tree), or file (blob) URL.");
    const tail = route.slice(1);
    const requested = branch.trim();
    const refParts = requested.split("/");
    const consumed = requested && refParts.every((part, i) => tail[i] === part) ? refParts.length : 1;
    const ref = requested || tail[0] || "";
    const path = tail.slice(consumed).join("/");
    if (route.length && !tail.length) throw new Error("The URL is missing its branch or revision.");
    return { kind: github ? "github" : "gitlab", origin: github ? "https://api.github.com" : url.origin, project, ref, path, file: ["blob", "raw"].includes(route[0]) };
}
function nextGitLabPageToken(link, origin) {
    const next = link?.split(",").find((part) => /;\s*rel\s*=\s*(?:"next"|next)(?:\s*;|\s*$)/i.test(part));
    const address = next?.match(/<([^>]+)>/)?.[1];
    let url;
    try { url = address && new URL(address, origin); } catch { throw new Error("Repository returned an invalid listing cursor."); }
    const token = url?.searchParams.get("page_token");
    if (next && (!url || url.origin !== origin || !url.pathname.endsWith("/repository/tree") || !token || token.length > 4096)) throw new Error("Repository returned an invalid listing cursor.");
    return token || "";
}
async function readRemoteBytes(response, maximum) {
    if (!response.ok) throw new Error(response.status === 429 || response.status === 403 ? "Access denied or API rate limit reached. Try later or choose a smaller public folder." : response.status === 404 ? "Repository, branch or path not found. If it is private, save a repository token." : `Repository request failed (${response.status}).`);
    if (Number(response.headers.get("content-length")) > maximum) throw new RemoteSizeError("Remote response exceeds the size limit. Choose a smaller folder or file.");
    const reader = response.body.getReader();
    const chunks = [];
    let size = 0;
    try {
        while (true) {
            const { done, value } = await reader.read();
            if (done) break;
            size += value.byteLength;
            if (size > maximum) throw new RemoteSizeError("Remote response exceeds the size limit. Choose a smaller folder or file.");
            chunks.push(value);
        }
    } finally { await reader.cancel(); }
    const bytes = new Uint8Array(size);
    let offset = 0;
    for (const chunk of chunks) { bytes.set(chunk, offset); offset += chunk.byteLength; }
    return bytes;
}
window.acornRemote = createRemoteLoader();
window.dispatchEvent?.(new Event("acorn-remote-ready"));
