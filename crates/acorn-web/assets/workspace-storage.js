function createWorkspaceStorage(storage = () => window.localStorage) {
    const key = "acorn.workspace.v1";
    const consent = "acorn.workspace-consent.v1";
    const limit = 17 * 1024 * 1024;
    let failed = false;
    const enabled = () => { try { return storage().getItem(consent) === "yes"; } catch { return false; } };
    const forget = () => { for (const name of [consent, key]) { try { storage().removeItem(name); } catch { /* Storage may be unavailable. */ } } };
    return {
        status() { try { return { enabled: enabled(), bytes: storage().getItem(key)?.length || 0, failed }; } catch { return { enabled: false, bytes: 0, failed }; } },
        restore() {
            if (!enabled()) return null;
            try { const raw = storage().getItem(key); const saved = raw && raw.length <= limit ? JSON.parse(raw) : null; return saved?.schema === 1 ? saved.workspace : null; } catch { return null; }
        },
        save(workspace) {
            if (!enabled()) return false;
            try {
                const raw = JSON.stringify({ schema: 1, workspace });
                if (raw.length > limit) throw new Error("Workspace snapshot exceeds the storage limit.");
                storage().setItem(key, raw);
                failed = false;
                return true;
            } catch { failed = true; forget(); return false; }
        },
        enable(workspace) {
            try {
                storage().setItem(consent, "yes");
                return this.save(workspace);
            } catch { failed = true; forget(); return false; }
        },
        delete() {
            try { storage().removeItem(key); storage().removeItem(consent); failed = false; return true; } catch { return false; }
        },
    };
}
window.acornWorkspaceStorage = createWorkspaceStorage();
window.dispatchEvent?.(new Event("acorn-workspace-storage-ready"));
