const dashboardTiles = {
    portfolio: ["activities", "organizations", "contributors", "top-topics", "statuses", "identifiers"],
};
function createDashboardLayoutStorage(storage = () => window.localStorage) {
    const key = "acorn.dashboard.layout.v1";
    const defaults = (page) => dashboardTiles[page] ? { order: [...dashboardTiles[page]], hidden: [] } : null;
    const normalize = (page, value) => {
        const known = dashboardTiles[page];
        if (!known) return null;
        const supplied = Array.isArray(value?.order) ? value.order.filter((id) => known.includes(id)) : [];
        const order = [...new Set([...supplied, ...known])];
        const hidden = [...new Set(Array.isArray(value?.hidden) ? value.hidden.filter((id) => known.includes(id)) : [])];
        const positions = Object.fromEntries(known.filter((id) => {
            const cell = value?.positions?.[id];
            const span = ["activities", "organizations", "contributors"].includes(id) ? 1 : 2;
            return Number.isInteger(cell?.row) && cell.row >= 1 && cell.row <= 8
                && Number.isInteger(cell?.column) && cell.column >= 1 && cell.column + span <= 5;
        }).map((id) => [id, { row: value.positions[id].row, column: value.positions[id].column }]));
        return Object.keys(positions).length ? { order, hidden, positions } : { order, hidden };
    };
    const read = () => {
        try {
            const raw = storage().getItem(key);
            if (!raw || raw.length > 4096) return { schema: 1, pages: {} };
            const saved = JSON.parse(raw);
            if (saved?.schema !== 1) return null;
            return { schema: 1, pages: saved.pages && typeof saved.pages === "object" && !Array.isArray(saved.pages) ? saved.pages : {} };
        } catch { return { schema: 1, pages: {} }; }
    };
    const load = (page) => normalize(page, read()?.pages?.[page] ?? defaults(page));
    const save = (page, value) => {
        const layout = normalize(page, value);
        const saved = read();
        if (!layout || !saved) return false;
        const pages = Object.fromEntries(Object.keys(dashboardTiles).filter((name) => saved.pages[name]).map((name) => [name, normalize(name, saved.pages[name])]));
        try { storage().setItem(key, JSON.stringify({ schema: 1, pages: { ...pages, [page]: layout } })); return true; }
        catch { return false; }
    };
    const reset = (page) => {
        const saved = read();
        if (!dashboardTiles[page] || !saved) return false;
        const pages = Object.fromEntries(Object.keys(dashboardTiles).filter((name) => name !== page && saved.pages[name]).map((name) => [name, normalize(name, saved.pages[name])]));
        try {
            if (Object.keys(pages).length) storage().setItem(key, JSON.stringify({ schema: 1, pages }));
            else storage().removeItem(key);
            return true;
        } catch { return false; }
    };
    const clear = () => {
        try { storage().removeItem(key); return true; }
        catch { return false; }
    };
    return { clear, load, reset, save };
}
if (typeof window !== "undefined") {
    window.acornDashboardLayout = createDashboardLayoutStorage();
    if (typeof window.dispatchEvent === "function") window.dispatchEvent(new Event("acorn-dashboard-layout-ready"));
}
