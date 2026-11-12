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
        // This raster/ellipsoid viewer does not expose 3D Tiles, KTX, splats or ZIP imports.
        for (const decoder of ["basis_transcoder.wasm", "draco_decoder.wasm", "zip-module.wasm", "wasm_splats_bg.wasm"]) rmSync(resolve(target, decoder), { force: true });
    }
}
export default defineConfig(({ command }) => ({
    base: "./",
    define: { "process.env.NODE_ENV": JSON.stringify(command === "serve" ? "development" : "production") },
    plugins: [react()],
    build: {
        lib: { entry: resolve("src/main.tsx"), formats: ["es"], fileName: () => "map.js", cssFileName: "map" },
        sourcemap: false,
        assetsInlineLimit: 0,
    },
    server: { cors: { origin: frontendOrigins } },
}));
