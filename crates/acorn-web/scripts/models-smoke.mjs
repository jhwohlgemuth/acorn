// SPDX-License-Identifier: MIT
// Render the checked-in supercomputer with the installed Cesium, without external requests.
import assert from "node:assert/strict";
import { readFile, mkdir } from "node:fs/promises";
import { createServer } from "node:http";
import { dirname, extname, resolve, sep } from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const screenshot = resolve(process.argv[2] ?? `${root}/../../output/designs/cesium-objects/supercomputer-preview.png`);
const html = `<!doctype html><html lang="en"><head><meta charset="utf-8"><title>ACORN supercomputer model inspection</title>
<script>window.CESIUM_BASE_URL='/cesium/';</script><script src="/cesium/Cesium.js"></script>
<link rel="stylesheet" href="/cesium/Widgets/widgets.css"><style>
html,body,#view{width:100%;height:100%;margin:0;background:#101923}header{position:absolute;top:36px;left:44px;color:#f4eddd;font:18px system-ui;pointer-events:none}h1{font-size:30px;margin:0 0 8px}p{color:#9daebc;margin:0}
</style></head><body><div id="view"></div><header><h1>ACORN / SUPERCOMPUTER</h1><p>Three-rack research miniature · actual GLB rendered in Cesium</p></header><script>
window.modelReady=false;
const C=Cesium;
C.Ion.defaultAccessToken='';
const viewer=new C.Viewer('view',{baseLayer:false,globe:false,skyBox:false,skyAtmosphere:false,sun:false,moon:false,animation:false,timeline:false,baseLayerPicker:false,geocoder:false,homeButton:false,sceneModePicker:false,navigationHelpButton:false,fullscreenButton:false,infoBox:false});
viewer.scene.backgroundColor=C.Color.fromCssColorString('#101923');
viewer.scene.highDynamicRange=false;
viewer.scene.light=new C.DirectionalLight({direction:new C.Cartesian3(-0.4,-0.6,-1),intensity:3});
const frame=C.Transforms.eastNorthUpToFixedFrame(C.Cartesian3.fromDegrees(-84.3,35.9));
C.Model.fromGltfAsync({url:'/models/research/supercomputer.glb',modelMatrix:frame}).then(model=>{
 viewer.scene.primitives.add(model);
 model.readyEvent.addEventListener(()=>{
   window.inspectView=(heading)=>viewer.camera.lookAtTransform(frame,new C.HeadingPitchRange(heading,-0.38,8.6));
   window.inspectView(Math.PI+0.55);
   window.modelReady=true;
 });
 model.errorEvent.addEventListener(error=>{window.modelFailure=String(error)});
}).catch(error=>{window.modelFailure=String(error)});
</script></body></html>`;
const types = { ".css": "text/css", ".glb": "model/gltf-binary", ".js": "text/javascript", ".json": "application/json", ".png": "image/png", ".wasm": "application/wasm" };
const server = createServer(async (request, response) => {
    try {
        const path = decodeURIComponent(new URL(request.url, "http://localhost").pathname);
        if (path === "/") {
            response.writeHead(200, { "Content-Type": "text/html" });
            response.end(html);
        } else {
            const mount = path.startsWith("/cesium/") ? ["/cesium/", "geospatial/node_modules/cesium/Build/Cesium"] : ["/models/", "assets/models"];
            assert.ok(path.startsWith(mount[0]));
            const base = resolve(root, mount[1]);
            const file = resolve(base, path.slice(mount[0].length));
            assert.ok(file.startsWith(`${base}${sep}`));
            const content = await readFile(file);
            response.writeHead(200, { "Content-Type": types[extname(file)] ?? "application/octet-stream" });
            response.end(content);
        }
    } catch {
        response.writeHead(404);
        response.end();
    }
});
await new Promise((ready) => server.listen(0, "127.0.0.1", ready));
const origin = `http://127.0.0.1:${server.address().port}`;
let browser;
try {
    browser = await chromium.launch({ headless: true });
    const page = await browser.newPage({ viewport: { width: 1200, height: 900 }, deviceScaleFactor: 1 });
    const failures = [];
    const external = [];
    page.on("pageerror", (error) => failures.push(String(error)));
    page.on("response", (response) => { if (response.status() >= 400) failures.push(`${response.status()} ${response.url()}`); });
    await page.route("**/*", (route) => {
        if (route.request().url().startsWith(`${origin}/`)) void route.continue();
        else { external.push(route.request().url()); void route.abort(); }
    });
    await page.goto(origin);
    await page.waitForFunction(() => window.modelReady || window.modelFailure, undefined, { timeout: 30000 });
    assert.equal(await page.evaluate(() => window.modelFailure), undefined);
    await mkdir(dirname(screenshot), { recursive: true });
    for (const [name, heading] of [["front", Math.PI + 0.55], ["rear", 0.55]]) {
        await page.evaluate((angle) => window.inspectView(angle), heading);
        await page.evaluate(() => new Promise((done) => requestAnimationFrame(() => requestAnimationFrame(done))));
        await page.screenshot({ path: name === "front" ? screenshot : screenshot.replace(/\.png$/, "-rear.png") });
    }
    assert.deepEqual(external, [], "Model inspection requested external resources");
    assert.deepEqual(failures, [], "Model inspection reported browser errors");
    console.log(`Cesium supercomputer load and front/rear renders passed; no external requests. Preview: ${screenshot}`);
} finally {
    if (browser) await browser.close();
    await new Promise((done) => server.close(done));
}
