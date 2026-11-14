export type Marker = { id: string; name: string; references: number; location: null | { latitude: number; longitude: number; country: string } };
export type State = { markers: Marker[]; selected: string | null; theme: "dark" | "light" };

export function validateState(state: State): State {
    if (!state || !Array.isArray(state.markers) || state.markers.length > 1000 || !["dark", "light"].includes(state.theme)) throw new Error("Invalid map state");
    const ids = new Set();
    for (const marker of state.markers) {
        if (!/^https:\/\/ror.org\/0[a-z0-9]{8}$/.test(marker.id) || ids.has(marker.id) || typeof marker.name !== "string" || marker.name.length > 1000 || !Number.isSafeInteger(marker.references) || marker.references < 0) throw new Error("Invalid map marker");
        ids.add(marker.id);
        if (marker.location && (!Number.isFinite(marker.location.latitude) || !Number.isFinite(marker.location.longitude) || Math.abs(marker.location.latitude) > 90 || Math.abs(marker.location.longitude) > 180)) throw new Error("Invalid map coordinates");
    }
    if (state.selected !== null && !ids.has(state.selected)) throw new Error("Selection is outside the marker set");
    return structuredClone(state);
}
export function createBridge(render: (state: State) => void, destroy: () => void, notify: (event: { type: string; id?: string; message?: string }) => void) {
    let current: State | null = null;
    let disposed = false;
    return {
        update(state: State) { if (!disposed) { current = validateState(state); render(current); } },
        select(id: string) { if (!disposed && current?.markers.some((marker) => marker.id === id)) notify({ type: "select", id }); },
        error(message: string) { if (!disposed) notify({ type: "error", message }); },
        dispose() { if (!disposed) { disposed = true; current = null; destroy(); } },
    };
}
