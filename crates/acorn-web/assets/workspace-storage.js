function createWorkspaceStorage(storage = () => window.localStorage) {
    const key = "acorn.workspace.v1";
    const limit = 33 * 1024 * 1024;
    let failed = false;
    return {
        status() { try { return { bytes: storage().getItem(key)?.length || 0, failed }; } catch { return { bytes: 0, failed }; } },
        restore() {
            try { const raw = storage().getItem(key); const saved = raw && raw.length <= limit ? JSON.parse(raw) : null; return saved?.schema === 1 ? saved.workspace : null; } catch { return null; }
        },
        save(workspace) {
            if (!workspace.files?.length) return true;
            try {
                const raw = JSON.stringify({ schema: 1, workspace });
                if (raw.length > limit) throw new Error("Workspace snapshot exceeds the storage limit.");
                storage().setItem(key, raw);
                failed = false;
                return true;
            } catch { failed = true; return false; }
        },
        delete() {
            try { storage().removeItem(key); storage().removeItem("acorn.workspace-consent.v1"); failed = false; return true; } catch { return false; }
        },
    };
}
window.acornWorkspaceStorage = createWorkspaceStorage();
window.dispatchEvent?.(new Event("acorn-workspace-storage-ready"));
