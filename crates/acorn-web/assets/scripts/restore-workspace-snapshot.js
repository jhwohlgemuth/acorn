// SPDX-License-Identifier: MIT
if (!window.acornWorkspaceStorage) {
    await new Promise(resolve => window.addEventListener('acorn-workspace-storage-ready', resolve, { once: true }));
}
const storage = window.acornWorkspaceStorage;
const workspace = await storage.restore();
return { workspace: workspace, recoveryTimedOut: storage.recoveryTimedOut };
