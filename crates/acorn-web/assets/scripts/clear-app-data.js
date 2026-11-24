// SPDX-License-Identifier: MIT
window.acornWorkspace?.cancel();
window.acornRemote?.cancel();
let workspaceDeleted = false;
let remoteDeleted = false;
let layoutDeleted = false;
let tourDeleted = false;
try { workspaceDeleted = await window.acornWorkspaceStorage?.delete() ?? false; } catch {}
try { remoteDeleted = window.acornRemote?.clearData() ?? false; } catch {}
try { layoutDeleted = window.acornDashboardLayout?.clear() ?? false; } catch {}
try { tourDeleted = window.acornChrome?.tour.reset() ?? false; } catch {}
return [workspaceDeleted, remoteDeleted, layoutDeleted, tourDeleted];
