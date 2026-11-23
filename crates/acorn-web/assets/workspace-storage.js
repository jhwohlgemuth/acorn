function createWorkspaceStorage(storage = () => window.localStorage, indexedDB = () => window.indexedDB, manager = () => navigator.storage, restoreTimeoutMs = 8000) {
    const legacyKey = "acorn.workspace.v1";
    const fallbackKey = "acorn.workspace.fallback.v1";
    const legacyLimit = 33 * 1024 * 1024;
    let database;
    let pending = Promise.resolve();
    let knownFiles = new Map();
    let knownRevision = null;
    let fallbackActive = false;
    let bytes = 0;
    let failed = false;
    let recoveryTimedOut = false;
    const request = (operation) => new Promise((resolve, reject) => {
        operation.onsuccess = () => resolve(operation.result);
        operation.onerror = () => reject(operation.error);
    });
    const complete = (transaction) => new Promise((resolve, reject) => {
        transaction.oncomplete = () => resolve();
        transaction.onabort = () => reject(transaction.error);
        transaction.onerror = () => reject(transaction.error);
    });
    const open = () => {
        if (!database) database = new Promise((resolve, reject) => {
            const operation = indexedDB().open("acorn.workspace", 2);
            let blocked = false;
            operation.onupgradeneeded = () => {
                const db = operation.result;
                if (!db.objectStoreNames.contains("state")) db.createObjectStore("state");
                if (!db.objectStoreNames.contains("files")) db.createObjectStore("files", { keyPath: "path" });
            };
            operation.onblocked = () => { blocked = true; reject(new Error("Workspace storage is blocked by another tab.")); };
            operation.onsuccess = () => {
                if (blocked) { operation.result.close(); return; }
                operation.result.onversionchange = () => operation.result.close();
                resolve(operation.result);
            };
            operation.onerror = () => reject(operation.error);
        }).catch((error) => { database = undefined; throw error; });
        return database;
    };
    const snapshot = (key) => {
        try {
            const raw = storage().getItem(key);
            const saved = raw && raw.length <= legacyLimit ? JSON.parse(raw) : null;
            return saved?.schema === 1 ? saved.workspace : null;
        } catch { return null; }
    };
    const restoreLocal = (key) => {
        const workspace = snapshot(key);
        try { bytes = workspace ? storage().getItem(key).length : 0; } catch {}
        return workspace;
    };
    const imageRecord = (file) => {
        const match = /^data:(image\/(?:png|jpeg|webp));base64,(.*)$/s.exec(file.content);
        if (!match) return file;
        const data = Uint8Array.from(atob(match[2]), (character) => character.charCodeAt(0));
        return { path: file.path, content: new Blob([data], { type: match[1] }) };
    };
    const imageContent = async (blob) => {
        const data = new Uint8Array(await blob.arrayBuffer());
        const parts = [];
        for (let offset = 0; offset < data.length; offset += 24576) {
            parts.push(btoa(String.fromCharCode(...data.subarray(offset, offset + 24576))));
        }
        return `data:${blob.type};base64,${parts.join("")}`;
    };
    const saveIndexed = async (workspace, db) => {
        const files = new Map(workspace.files.map((file) => [file.path, file.content]));
        const nextBytes = workspace.files.reduce((total, file) => total + file.content.length, 0);
        const transaction = db.transaction(["state", "files"], "readwrite");
        const done = complete(transaction);
        const state = transaction.objectStore("state");
        let current;
        try { current = await request(state.get("workspace")); }
        catch (error) { await done.catch(() => {}); throw error; }
        const revision = current?.schema === 2 ? current.revision ?? "unversioned" : null;
        if (revision !== knownRevision) {
            transaction.abort();
            await done.catch(() => {});
            throw new Error("Workspace changed in another tab.");
        }
        const nextRevision = globalThis.crypto?.randomUUID?.() ?? `${Date.now()}:${Math.random()}`;
        try {
            const stored = transaction.objectStore("files");
            for (const path of knownFiles.keys()) if (!files.has(path)) stored.delete(path);
            for (const file of workspace.files) if (knownFiles.get(file.path) !== file.content) stored.put(imageRecord(file));
            const { files: omitted, ...metadata } = workspace;
            state.put({ schema: 2, revision: nextRevision, workspace: metadata, bytes: nextBytes }, "workspace");
        } catch (error) {
            transaction.abort();
            await done.catch(() => {});
            throw error;
        }
        await done;
        knownFiles = files;
        knownRevision = nextRevision;
        bytes = nextBytes;
        try { storage().removeItem(legacyKey); } catch {}
    };
    const saveFallback = (workspace) => {
        const raw = JSON.stringify({ schema: 1, workspace });
        if (raw.length > legacyLimit) throw new Error("Workspace snapshot exceeds the storage limit.");
        storage().setItem(fallbackKey, raw);
        fallbackActive = true;
        bytes = raw.length;
        try { storage().removeItem(legacyKey); } catch {}
    };
    const save = (workspace) => {
        const work = pending.then(async () => {
            if (recoveryTimedOut) { failed = true; return false; }
            if (!workspace.files?.length) return true;
            try {
                if (fallbackActive) saveFallback(workspace);
                else {
                    let db;
                    try { if (indexedDB()) db = await open(); } catch {}
                    if (db) await saveIndexed(workspace, db);
                    else saveFallback(workspace);
                }
                failed = false;
                return true;
            } catch { failed = true; return false; }
        });
        pending = work.then(() => {});
        return work;
    };
    return {
        get recoveryTimedOut() { return recoveryTimedOut; },
        async status() {
            let usage = 0;
            let quota = 0;
            try { ({ usage = 0, quota = 0 } = await manager()?.estimate() || {}); } catch {}
            return { bytes, failed, usage, quota };
        },
        async restore() {
            const fallback = restoreLocal(fallbackKey);
            if (fallback) {
                fallbackActive = true;
                return fallback;
            }
            const timeoutError = new Error("Workspace recovery timed out.");
            try {
                if (!indexedDB()) return restoreLocal(legacyKey);
                let timer;
                const timeout = new Promise((_, reject) => {
                    timer = setTimeout(() => reject(timeoutError), restoreTimeoutMs);
                });
                let saved, entries;
                try {
                    const db = await Promise.race([open(), timeout]);
                    const transaction = db.transaction(["state", "files"], "readonly");
                    const done = complete(transaction);
                    const state = request(transaction.objectStore("state").get("workspace"));
                    const files = request(transaction.objectStore("files").getAll());
                    [saved, entries] = await Promise.race([Promise.all([state, files, done]), timeout]);
                }
                finally { clearTimeout(timer); }
                if (recoveryTimedOut) return null;
                if (saved?.schema === 2) {
                    const restored = await Promise.all(entries.map(async (file) => ({ path: file.path, content: file.content instanceof Blob ? await imageContent(file.content) : file.content })));
                    knownFiles = new Map(restored.map((file) => [file.path, file.content]));
                    knownRevision = saved.revision ?? "unversioned";
                    bytes = saved.bytes;
                    return { ...saved.workspace, files: restored };
                }
                const workspace = restoreLocal(legacyKey);
                if (workspace?.files?.length) await save(workspace);
                return workspace;
            } catch (error) {
                if (error === timeoutError) {
                    recoveryTimedOut = true;
                    failed = true;
                }
                return restoreLocal(legacyKey);
            }
        },
        save,
        delete() {
            const work = pending.then(async () => {
                let indexedCleared = true;
                try {
                    if (indexedDB()) {
                        const db = await open();
                        const transaction = db.transaction(["state", "files"], "readwrite");
                        const done = complete(transaction);
                        try {
                            transaction.objectStore("state").clear();
                            transaction.objectStore("files").clear();
                        } catch (error) {
                            transaction.abort();
                            await done.catch(() => {});
                            throw error;
                        }
                        await done;
                    }
                } catch { indexedCleared = false; }
                try {
                    storage().removeItem(fallbackKey);
                    storage().removeItem(legacyKey);
                    storage().removeItem("acorn.workspace-consent.v1");
                    knownFiles = new Map();
                    knownRevision = null;
                    fallbackActive = false;
                    bytes = 0;
                    failed = false;
                    return indexedCleared;
                } catch { return false; }
            });
            pending = work.then(() => {});
            return work;
        },
    };
}
window.acornWorkspaceStorage = createWorkspaceStorage();
window.dispatchEvent?.(new Event("acorn-workspace-storage-ready"));
