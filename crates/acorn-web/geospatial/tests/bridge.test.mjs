import assert from "node:assert/strict";
import { performance } from "node:perf_hooks";
import { test } from "node:test";
import { createBridge, validateState } from "../src/bridge.ts";
import { locationIndex } from "../../scripts/build-ror-location-index.mjs";

const project = { id: "project-alpha", title: "Alpha", activity_count: 2 };
const marker = { id: "https://ror.org/01qz5mb56", name: "<b>Literal name</b>", references: 2, project_ids: [project.id], location: { latitude: 35, longitude: -84, country: "United States" } };
const state = { version: 1, projects: [project], markers: [marker], selected: null, theme: "dark" };
test("500 projects and 1000 marker DTOs remain bounded and retain selection", () => {
    const projects = Array.from({ length: 500 }, (_, number) => ({ id: `project-${number}`, title: `Project ${number}`, activity_count: 1 }));
    const markers = Array.from({ length: 1000 }, (_, number) => ({
        id: `https://ror.org/0${number.toString(36).padStart(8, "0")}`,
        name: `Organization ${number}`,
        references: 1,
        project_ids: [projects[Math.floor(number / 2)].id],
        location: { latitude: number % 90, longitude: -(number % 180), country: "United States" },
    }));
    const maximum = { version: 1, projects, markers, selected: markers.at(-1).id, theme: "dark" };
    const started = performance.now();
    const copy = validateState(maximum);
    const elapsed = performance.now() - started;
    const bytes = Buffer.byteLength(JSON.stringify(maximum));
    console.log(`ABW-18 bridge: 500 projects, 1000 markers, ${bytes} bytes, ${elapsed.toFixed(2)} ms`);
    assert.ok(bytes < 1_000_000);
    assert.equal(copy.markers.length, 1000);
    assert.equal(copy.selected, markers.at(-1).id);
    assert.equal(copy.markers[999].project_ids[0], projects[499].id);
    const shared = projects.map((project) => project.id);
    const dense = { ...maximum, markers: markers.map((marker) => ({ ...marker, project_ids: shared })) };
    const denseBytes = Buffer.byteLength(JSON.stringify(dense));
    assert.ok(denseBytes < 8_000_000, "Even full project overlap stays within the bridge budget");
    assert.equal(validateState(dense).markers.at(-1).project_ids.length, 500);
    console.log(`ABW-18 dense bridge: 500000 memberships, ${denseBytes} bytes`);
});
test("map bridge bounds its DTO and copies caller-owned state", () => {
    const copy = validateState(state);
    copy.markers[0].name = "changed";
    copy.projects[0].title = "changed";
    assert.equal(marker.name, "<b>Literal name</b>");
    assert.equal(project.title, "Alpha");
    for (const bad of [
        { ...state, version: 2 },
        { ...state, projects: null },
        { ...state, projects: Array(501).fill(project) },
        { ...state, projects: [project, project] },
        { ...state, projects: [{ ...project, activity_count: 0 }] },
        { ...state, markers: Array(1001).fill(marker) },
        { ...state, selected: "unknown" },
        { ...state, theme: "unknown" },
        { ...state, markers: [marker, marker] },
        { ...state, markers: [{ ...marker, references: -1 }] },
        { ...state, markers: [{ ...marker, project_ids: ["unknown"] }] },
        { ...state, markers: [{ ...marker, project_ids: [project.id, project.id] }] },
        { ...state, markers: [{ ...marker, location: { latitude: 91, longitude: 0 } }] },
        { ...state, markers: [{ ...marker, location: { latitude: NaN, longitude: 0 } }] },
    ]) assert.throws(() => validateState(bad));
});
test("mount updates, selection, error and disposal keep table ownership outside the map", () => {
    const renders = [], events = [];
    let destroyed = 0;
    const bridge = createBridge((state) => renders.push(state), () => destroyed++, (event) => events.push(event));
    bridge.update(state);
    bridge.select(marker.id);
    bridge.select("unknown");
    bridge.update({ ...state, selected: marker.id, theme: "light" });
    bridge.error("WebGL unavailable");
    bridge.dispose(); bridge.dispose(); bridge.select(marker.id); bridge.update(state); bridge.error("late");
    assert.equal(renders.length, 2);
    assert.equal(renders[1].theme, "light");
    assert.equal(destroyed, 1);
    assert.deepEqual(events, [{ type: "select", id: marker.id }, { type: "error", message: "WebGL unavailable" }]);
});
test("ROR extraction is deterministic and retains locationless organizations", () => {
    const record = (id, locations) => ({ id, names: [{ value: "Example", types: ["ror_display"] }], locations });
    const a = record("https://ror.org/01qz5mb56", [{ geonames_details: { country_name: "US", lat: 91, lng: 0 } }, { geonames_details: { country_name: "US", lat: 35, lng: -84 } }]);
    const b = record("https://ror.org/04aj4c181", []);
    assert.deepEqual(locationIndex([a, b]), locationIndex([b, a]));
    assert.equal(locationIndex([a])[a.id].locations.length, 1);
    assert.deepEqual(locationIndex([b])[b.id].locations, []);
});
test("standalone ROR pins have no activity references and remain selectable", () => {
    const standalone = { ...state, markers: [{ ...marker, references: 0, project_ids: [] }], selected: marker.id };
    assert.deepEqual(validateState(standalone), standalone);
});
