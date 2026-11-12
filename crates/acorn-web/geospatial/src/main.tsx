import "@vitejs/plugin-react/preamble";
import { Ion } from "cesium";
import { createRoot } from "react-dom/client";
import { createBridge, type State } from "./bridge";
import { MapBoundary, MapView } from "./map";
import "cesium/Build/Cesium/Widgets/widgets.css";
import "./map.css";

declare global { var CESIUM_BASE_URL: string; }

export function mount(host: HTMLElement, initial: State, notify: (event: unknown) => void) {
    const probe = document.createElement("canvas").getContext("webgl2");
    if (!probe) {
        notify({ type: "error", message: "The map is unavailable because WebGL 2 is disabled. All organization locations remain in the table." });
        return { update() {}, dispose() {} };
    }
    probe.getExtension("WEBGL_lose_context")?.loseContext();
    const base = import.meta.env.DEV ? new URL(/* @vite-ignore */ "/cesium/", import.meta.url).href : new URL(/* @vite-ignore */ "./cesium/", import.meta.url).href;
    globalThis.CESIUM_BASE_URL = base;
    Ion.defaultAccessToken = "";
    const root = createRoot(host);
    const bridge = createBridge((state) => root.render(<MapBoundary onError={bridge.error}><MapView state={state} base={base} select={bridge.select} error={bridge.error} /></MapBoundary>), () => root.unmount(), notify);
    bridge.update(initial);
    return bridge;
}
