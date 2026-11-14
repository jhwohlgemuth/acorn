const remoteLimits = { entries: 10000, file: 262144, files: 500, listing: 5242880, total: 16777216 };
function parseRemoteBucket(input, branch = "") {
    const value = input.trim();
    const url = new URL(/^[\w.-]+\/[\w.-]+$/.test(value) ? `https://github.com/${value}` : value);
    if (url.protocol !== "https:" || url.username || url.password || url.search || url.hash || url.port || !url.hostname.includes(".") || /^(localhost|127\.|10\.|192\.168\.|169\.254\.|172\.(1[6-9]|2\d|3[01])\.)/.test(url.hostname) || url.hostname.startsWith("[")) throw new Error("Use a public HTTPS GitHub or GitLab repository URL without credentials or query parameters.");
    const parts = url.pathname.split("/").filter(Boolean).map(decodeURIComponent);
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
async function readRemoteBytes(response, maximum) {
    if (!response.ok) throw new Error(response.status === 429 || response.status === 403 ? "Access denied or API rate limit reached. Try later or choose a smaller public folder." : response.status === 404 ? "Repository, branch or path not found. Check the URL and public access." : `Repository request failed (${response.status}).`);
    if (Number(response.headers.get("content-length")) > maximum) throw new Error("Remote response exceeds the size limit. Choose a smaller folder or file.");
    const reader = response.body.getReader();
    const chunks = [];
    let size = 0;
    try {
        while (true) {
            const { done, value } = await reader.read();
            if (done) break;
            size += value.byteLength;
            if (size > maximum) throw new Error("Remote response exceeds the size limit. Choose a smaller folder or file.");
            chunks.push(value);
        }
    } finally { await reader.cancel(); }
    const bytes = new Uint8Array(size);
    let offset = 0;
    for (const chunk of chunks) { bytes.set(chunk, offset); offset += chunk.byteLength; }
    return bytes;
}
async function loadRemoteBucket(source, signal, fetcher = fetch) {
    const base = source.kind === "github" ? `${source.origin}/repos/${source.project.split("/").map(encodeURIComponent).join("/")}` : `${source.origin}/api/v4/projects/${encodeURIComponent(source.project)}`;
    const request = async (path, maximum = remoteLimits.listing) => {
        signal.throwIfAborted();
        return readRemoteBytes(await fetcher(base + path, { signal, credentials: "omit", redirect: "error", referrerPolicy: "no-referrer", headers: { Accept: "application/json" } }), maximum);
    };
    const json = async (path) => JSON.parse(new TextDecoder("utf-8", { fatal: true }).decode(await request(path)));
    const repository = await json("");
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
        for (let page = 1; page <= 101; page += 1) {
            const batch = await json(`/repository/tree?recursive=true&per_page=100&page=${page}&ref=${revision}&path=${encodeURIComponent(source.path)}`);
            if (!Array.isArray(batch)) throw new Error("Invalid repository listing.");
            entries.push(...batch);
            if (entries.length > remoteLimits.entries) throw new Error("Folder contains too many entries. Choose a smaller folder.");
            if (batch.length < 100) break;
        }
    }
    const inside = (path) => !source.path || path === source.path || (!source.file && path.startsWith(source.path + "/"));
    const candidates = entries.filter((entry) => entry.type === "blob" && entry.mode !== "120000" && typeof entry.path === "string" && inside(entry.path));
    const accepted = candidates.filter((entry) => window.acornWorkspace.validPath(entry.path) && (entry.size === undefined || entry.size <= remoteLimits.file)).sort((a, b) => a.path.localeCompare(b.path));
    if (accepted.length > remoteLimits.files) throw new Error("More than 500 supported files. Choose a smaller folder.");
    const files = [];
    let total = 0;
    for (const entry of accepted) {
        signal.throwIfAborted();
        let bytes;
        if (source.kind === "github") {
            if (!/^[a-f0-9]{40,64}$/i.test(entry.sha || "")) throw new Error("Invalid repository file identity.");
            const blob = JSON.parse(new TextDecoder().decode(await request(`/git/blobs/${entry.sha}`, remoteLimits.file * 2)));
            if (blob.encoding !== "base64" || typeof blob.content !== "string") throw new Error("Unsupported repository file encoding.");
            bytes = Uint8Array.from(atob(blob.content.replace(/\s/g, "")), (char) => char.charCodeAt(0));
        } else {
            bytes = await request(`/repository/files/${encodeURIComponent(entry.path)}/raw?ref=${revision}`, remoteLimits.file);
        }
        total += bytes.byteLength;
        if (bytes.byteLength > remoteLimits.file || total > remoteLimits.total) throw new Error("Bucket exceeds the 256 KiB per-file or 16 MiB total limit.");
        files.push({ path: entry.path, content: new TextDecoder("utf-8", { fatal: true }).decode(bytes) });
    }
    signal.throwIfAborted();
    if (!files.length) throw new Error("No supported research files found at this location.");
    return { files, skipped: candidates.length - accepted.length };
}
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
        clear() { try { storage().removeItem(key); } catch { /* Storage may be disabled. Loading remains available. */ } },
        read,
        remember(input, branch) {
            const saved = read();
            const next = { repositories: recent([input, ...saved.repositories]), refs: recent([branch, ...saved.refs]) };
            try { storage().setItem(key, JSON.stringify(next)); } catch { /* History is optional when storage is unavailable or full. */ }
        },
    };
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
async function loadRemoteFile(input, maximum, signal, fetcher = fetch) {
    const url = parseFileUrl(input);
    if (!Number.isSafeInteger(maximum) || maximum < 1 || maximum > remoteLimits.file) throw new Error("Invalid file size limit.");
    signal.throwIfAborted();
    const response = await fetcher(url.href, { signal, credentials: "omit", redirect: "follow", referrerPolicy: "no-referrer" });
    if (!response.ok) throw new Error(`Could not load this file (HTTP ${response.status}). Check the URL and access permissions.`);
    if (response.url) parseFileUrl(response.url);
    const bytes = await readRemoteBytes(response, maximum);
    signal.throwIfAborted();
    let content;
    try { content = new TextDecoder("utf-8", { fatal: true }).decode(bytes); }
    catch { throw new Error("Choose a UTF-8 text file."); }
    const name = decodeURIComponent(url.pathname.split("/").pop() || "download.txt");
    const path = !name || name === "." || name === ".." || /[\\/\x00-\x1f\x7f]/.test(name) ? "download.txt" : name;
    const extension = /json/i.test(response.headers.get("content-type") || "") ? ".json" : /yaml/i.test(response.headers.get("content-type") || "") ? ".yaml" : ".txt";
    return { path: path.includes(".") ? path : path + extension, content };
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
                const file = await loadRemoteFile(input, maximum, controller.signal, fetcher);
                controller.signal.throwIfAborted();
                if (workspaceFile && !window.acornWorkspace.validPath(file.path)) throw new Error("Choose a JSON, YAML, Markdown or text file for this workspace.");
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
    const file = createRemoteFileLoader();
    const history = createBucketHistory();
    return {
        cancel() { active?.abort(); active = undefined; file.cancel(); },
        file,
        history,
        async load(input, branch) {
            window.acornWorkspace.cancel();
            const controller = new AbortController();
            active = controller;
            const timeout = setTimeout(() => controller.abort(), 120000);
            try {
                const source = parseRemoteBucket(input, branch);
                history.remember(input, branch);
                return await loadRemoteBucket(source, controller.signal);
            }
            catch (error) { throw new Error(controller.signal.aborted ? "Remote load cancelled or timed out. Your previous session is unchanged." : error instanceof TypeError ? "Could not load this public repository. Check the URL, network connection and the server’s CORS settings." : error.message); }
            finally { clearTimeout(timeout); if (active === controller) active = undefined; }
        },
    };
}
window.acornRemote = createRemoteLoader();
window.dispatchEvent?.(new Event("acorn-remote-ready"));
