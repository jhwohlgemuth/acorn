export type Marker = { id: string; name: string; references: number; project_ids: string[]; location: null | { latitude: number; longitude: number; country: string } };
export type Project = { id: string; title: string; activity_count: number };
export type State = { version?: 1; projects?: Project[]; markers: Marker[]; selected: string | null; theme: "dark" | "light" | "uswds" };

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
export function validateState(state: State): State {
    if (!state || (state.version !== undefined && state.version !== 1) || !Array.isArray(state.markers) || state.markers.length > 1000 || !["dark", "light", "uswds"].includes(state.theme)) throw new Error("Invalid map state");
    if (state.version === 1 && !Array.isArray(state.projects)) throw new Error("Missing map projects");
    const projects = state.projects ?? [];
    if (!Array.isArray(projects) || projects.length > 500) throw new Error("Invalid map projects");
    const projectIds = new Set<string>();
    for (const project of projects) {
        if (typeof project.id !== "string" || !project.id || project.id.length > 1000 || projectIds.has(project.id) || typeof project.title !== "string" || project.title.length > 1000 || !Number.isSafeInteger(project.activity_count) || project.activity_count < 1 || project.activity_count > 500) throw new Error("Invalid map project");
        projectIds.add(project.id);
    }
    const ids = new Set();
    for (const marker of state.markers) {
        if (!/^https:\/\/ror.org\/0[a-z0-9]{8}$/.test(marker.id) || ids.has(marker.id) || typeof marker.name !== "string" || marker.name.length > 1000 || !Number.isSafeInteger(marker.references) || marker.references < 0) throw new Error("Invalid map marker");
        ids.add(marker.id);
        if (!Array.isArray(marker.project_ids) || marker.project_ids.length > 500 || new Set(marker.project_ids).size !== marker.project_ids.length || marker.project_ids.some((id) => !projectIds.has(id))) throw new Error("Invalid marker projects");
        if (marker.location && (!Number.isFinite(marker.location.latitude) || !Number.isFinite(marker.location.longitude) || Math.abs(marker.location.latitude) > 90 || Math.abs(marker.location.longitude) > 180)) throw new Error("Invalid map coordinates");
    }
    if (state.selected !== null && !ids.has(state.selected)) throw new Error("Selection is outside the marker set");
    const copy = structuredClone(state);
    return { ...copy, version: 1, projects: copy.projects ?? [] };
}
