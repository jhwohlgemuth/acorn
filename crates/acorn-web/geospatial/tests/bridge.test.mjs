import assert from "node:assert/strict";
import { test } from "node:test";
import { createBridge, validateState } from "../src/bridge.ts";
import { locationIndex } from "../../scripts/build-ror-location-index.mjs";

const marker = { id: "https://ror.org/01qz5mb56", name: "<b>Literal name</b>", references: 2, location: { latitude: 35, longitude: -84, country: "United States" } };
const state = { markers: [marker], selected: null, theme: "dark" };
test("map bridge bounds its DTO and copies caller-owned state", () => {
    const copy = validateState(state);
    copy.markers[0].name = "changed";
    assert.equal(marker.name, "<b>Literal name</b>");
    for (const bad of [
        { ...state, markers: Array(1001).fill(marker) },
        { ...state, selected: "unknown" },
        { ...state, theme: "unknown" },
        { ...state, markers: [marker, marker] },
        { ...state, markers: [{ ...marker, references: -1 }] },
        { ...state, markers: [{ ...marker, location: { latitude: 91, longitude: 0 } }] },
        { ...state, markers: [{ ...marker, location: { latitude: NaN, longitude: 0 } }] },
    ]) assert.throws(() => validateState(bad));
});
test("standalone ROR pins have no activity references and remain selectable", () => {
    const standalone = { ...state, markers: [{ ...marker, references: 0 }], selected: marker.id };
    assert.deepEqual(validateState(standalone), standalone);
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
