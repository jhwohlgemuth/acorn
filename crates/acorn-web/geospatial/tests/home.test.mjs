import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";
import { Cartographic, Math as CesiumMath } from "cesium";
import { configureHome, ornlHome } from "../src/home.ts";

test("initial view and Home center on bundled ORNL coordinates without resetting to the global view", async () => {
    const index = JSON.parse(await readFile(new URL("../../assets/geospatial/ror-locations.json", import.meta.url)));
    const location = index.organizations["https://ror.org/01qz5mb56"].locations[0];
    assert.equal(ornlHome.latitude, location.latitude);
    assert.equal(ornlHome.longitude, location.longitude);
    const views = [];
    let home;
    const viewer = { camera: { setView: (view) => views.push(view), flyTo: (view) => views.push(view) }, homeButton: { viewModel: { command: { beforeExecute: { addEventListener: (callback) => { home = callback; } } } } } };
    configureHome(viewer);
    const initial = Cartographic.fromCartesian(views[0].destination);
    assert.ok(Math.abs(CesiumMath.toDegrees(initial.latitude) - location.latitude) < 1e-8);
    assert.ok(Math.abs(CesiumMath.toDegrees(initial.longitude) - location.longitude) < 1e-8);
    assert.ok(Math.abs(initial.height - 8000000) < 1e-6);
    assert.equal(viewer.homeButton.viewModel.tooltip, "Center on ORNL");
    try {
        for (const reduced of [false, true]) {
            globalThis.window = { matchMedia: () => ({ matches: reduced }) };
            const event = { cancel: false };
            home(event);
            assert.equal(event.cancel, true);
            assert.deepEqual(views.at(-1).destination, views[0].destination);
            assert.equal(views.at(-1).duration, reduced ? 0 : 1);
        }
    } finally { delete globalThis.window; }
});
