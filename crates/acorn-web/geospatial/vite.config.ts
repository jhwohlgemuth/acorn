import react from "@vitejs/plugin-react";
import { cpSync, mkdirSync, rmSync } from "node:fs";
import { resolve } from "node:path";
import { defineConfig } from "vite";

const frontend = new URL(process.env.ACORN_WEB_DEV_ORIGIN || "http://127.0.0.1:8080");
const frontendOrigins = ["127.0.0.1", "localhost"].map((hostname) => {
    const origin = new URL(frontend.origin);
    origin.hostname = hostname;
    return origin.origin;
});
for (const folder of ["Assets", "ThirdParty", "Widgets", "Workers"]) {
    const target = resolve("public/cesium", folder);
    mkdirSync(target, { recursive: true });
    cpSync(resolve("node_modules/cesium/Build/Cesium", folder), target, { recursive: true });
    if (folder === "ThirdParty") {
        // Optional OSM Buildings can use Draco-compressed 3D Tiles; KTX, splats and ZIP imports remain unavailable.
        for (const decoder of ["basis_transcoder.wasm", "zip-module.wasm", "wasm_splats_bg.wasm"]) rmSync(resolve(target, decoder), { force: true });
    }
}
cpSync(resolve("imagery/BlackMarble"), resolve("public/cesium/Assets/Textures/BlackMarble"), { recursive: true });
export default defineConfig(({ command }) => ({
    base: "./",
    define: { "process.env.NODE_ENV": JSON.stringify(command === "serve" ? "development" : "production") },
    plugins: [react()],
    build: {
        lib: { entry: resolve("src/main.tsx"), formats: ["es"], fileName: () => "map.js", cssFileName: "map" },
        sourcemap: false,
        assetsInlineLimit: 0,
    },
    // Library mode has no index.html for Vite's dependency scanner. Without an explicit entry, a cold cache
    // discovers Cesium/React on the first cross-origin import and reloads mid-load, failing that import.
    optimizeDeps: { entries: ["src/main.tsx"] },
    server: { cors: { origin: frontendOrigins }, warmup: { clientFiles: ["./src/main.tsx"] } },
}));
