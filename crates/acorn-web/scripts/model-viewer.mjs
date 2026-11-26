// SPDX-License-Identifier: MIT
import { readFile, realpath } from "node:fs/promises";
import { createServer } from "node:http";
import { dirname, extname, resolve, sep } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const types = { ".css": "text/css", ".glb": "model/gltf-binary", ".html": "text/html", ".js": "text/javascript", ".json": "application/json", ".png": "image/png", ".svg": "image/svg+xml", ".wasm": "application/wasm", ".webp": "image/webp" };

/** Serve only the viewer, local Cesium distribution and model assets on loopback. */
export async function startModelViewer(port = 8085, modelRoot = resolve(root, "assets/models")) {
    await readFile(resolve(root, "geospatial/node_modules/cesium/Build/Cesium/Cesium.js"));
    const server = createServer(async (request, response) => {
        try {
            const pathname = decodeURIComponent(new URL(request.url, "http://localhost").pathname);
            const shared = { "/acorn.css": ["assets", "app.css"], "/acorn-icon.svg": ["assets/generated/dark", "app-icon.svg"], "/acorn-icon-light.svg": ["assets/generated/light", "app-icon.svg"], "/acorn-icon-uswds.svg": ["assets/generated/uswds", "app-icon.svg"], "/uswds-sprite.svg": ["assets/vendor", "uswds-sprite.svg"] };
            const [directory, path] = Object.hasOwn(shared, pathname) ? shared[pathname] : pathname.startsWith("/cesium/")
                ? ["geospatial/node_modules/cesium/Build/Cesium", pathname.slice(8)]
                : pathname.startsWith("/models/")
                    ? [modelRoot, pathname.slice(8)]
                    : ["scripts/model-viewer", pathname === "/" ? "index.html" : pathname.slice(1)];
            const base = await realpath(resolve(root, directory));
            const file = await realpath(resolve(base, path));
            const allowed = file.startsWith(`${base}${sep}`)
                && (directory !== modelRoot || path === "catalog.json" || extname(path) === ".glb");
            if (!allowed) throw new Error("Outside viewer assets");
            if (!["GET", "HEAD"].includes(request.method)) {
                response.writeHead(405, { Allow: "GET, HEAD" });
                response.end();
            } else {
                const content = await readFile(file);
                response.writeHead(200, { "Cache-Control": "no-store", "Content-Type": types[extname(file)] ?? "application/octet-stream", "X-Content-Type-Options": "nosniff" });
                response.end(request.method === "HEAD" ? undefined : content);
            }
        } catch {
            response.writeHead(404, { "Content-Type": "text/plain" });
            response.end("Viewer asset not found");
        }
    });
    await new Promise((ready, reject) => {
        server.once("error", reject);
        server.listen(port, "127.0.0.1", ready);
    });
    return server;
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
    const port = Number(process.argv[2] ?? 8085);
    if (!Number.isInteger(port) || port < 0 || port > 65535) {
        console.error("Usage: node scripts/model-viewer.mjs [port: 0–65535]");
        process.exitCode = 1;
    } else {
        startModelViewer(port, process.argv[3] ? resolve(process.argv[3]) : undefined).then((server) => {
            console.log(`ACORN model viewer: http://127.0.0.1:${server.address().port}/`);
            console.log("Press Ctrl+C to stop. Refresh the page to pick up catalog changes.");
        }).catch((error) => {
            console.error(`Unable to start the model viewer: ${error.message}\nInstall geospatial dependencies with npm ci --prefix crates/acorn-web/geospatial if Cesium is missing.`);
            process.exitCode = 1;
        });
    }
}
