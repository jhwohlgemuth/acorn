// SPDX-License-Identifier: MIT
if (window.acornChrome) window.acornChrome.theme.reset();
try { sessionStorage.removeItem('acorn-sidebar-collapsed'); } catch {}
const tourCleared = window.acornChrome?.tour.reset() ?? false;
const layoutCleared = window.acornDashboardLayout?.clear() ?? false;
const graphCleared = window.acornGraphView?.clear() ?? false;
return tourCleared && layoutCleared && graphCleared;
