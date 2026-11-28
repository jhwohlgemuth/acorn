// SPDX-License-Identifier: MIT
/* global Cesium */
const assetPicker = document.getElementById("asset");
const status = document.getElementById("status");
const viewport = document.getElementById("viewport");
const viewName = document.getElementById("view-name");
const presets = {
    back: { heading: Math.PI / 2, pitch: -0.12 },
    front: { heading: Math.PI * 1.5, pitch: -0.12 },
    perspective: { heading: Math.PI * 1.5 - 0.55, pitch: -0.38 },
    top: { heading: Math.PI * 1.5, pitch: -Math.PI / 2 + 0.01 },
};

function describe(asset) {
    document.getElementById("category").textContent = asset.category.toUpperCase();
    document.getElementById("model-name").textContent = asset.label;
    document.getElementById("description").textContent = asset.provenance ?? "";
    const dimensions = asset.bounds?.max.map((value, axis) => (value - asset.bounds.min[axis]).toFixed(2)).join(" × ");
    const rows = [
        ["Triangles", asset.triangles?.toLocaleString() ?? "—"],
        ["File size", `${(asset.bytes / 1024).toFixed(1)} KiB`],
        ["Materials", asset.materials ?? "—"],
        ["Revision", asset.revision ?? "—"],
        ["Size (m)", dimensions ?? "—"],
        ["License", asset.license ?? "—"],
    ];
    document.getElementById("metadata").replaceChildren(...rows.flatMap(([label, value]) => {
        const term = document.createElement("dt");
        const description = document.createElement("dd");
        term.textContent = label;
        description.textContent = value;
        return [term, description];
    }));
    const download = document.getElementById("download");
    download.href = `/models/${asset.file}`;
    download.hidden = false;
}
function modelFolder(asset) {
    return asset.category === "marker" ? "markers" : asset.file.split("/")[0];
}
function report(message, error = false) {
    status.textContent = message;
    status.dataset.error = String(error);
}
async function start() {
    const response = await fetch("/models/catalog.json");
    if (!response.ok) throw new Error("The model catalog could not be loaded.");
    const catalog = await response.json();
    const valid = catalog.version === 1 && Array.isArray(catalog.assets) && catalog.assets.every((asset) =>
        [asset.id, asset.label, asset.category].every((value) => typeof value === "string" && value.length > 0)
        && /^[a-z0-9-]+\/[a-z0-9-]+\.glb$/.test(asset.file));
    if (!valid) throw new Error("The model catalog is invalid.");
    const assets = catalog.assets.map((asset) => asset.category === "research" && asset.id.endsWith("-marker")
        ? { ...asset, category: "marker" } : asset)
        .sort((left, right) => left.category.localeCompare(right.category) || left.label.localeCompare(right.label));
    if (assets.length === 0) throw new Error("No models yet. Add an asset to the catalog and refresh.");
    const initial = assets.find((asset) => asset.id === "research/supercomputer") ?? assets[0];
    describe(initial);
    Cesium.buildModuleUrl.setBaseUrl("/cesium/");
    Cesium.Ion.defaultAccessToken = "";
    const viewer = new Cesium.Viewer("viewport", {
        animation: false, baseLayer: false, baseLayerPicker: false, fullscreenButton: false,
        geocoder: false, globe: false, homeButton: false, infoBox: false, moon: false,
        navigationHelpButton: false, requestRenderMode: true, maximumRenderTimeChange: Infinity,
        sceneModePicker: false, selectionIndicator: false, skyAtmosphere: false, skyBox: false,
        sun: false, timeline: false,
    });
    viewer.scene.backgroundColor = Cesium.Color.fromCssColorString(getComputedStyle(document.body).getPropertyValue("--canvas").trim());
    viewer.scene.highDynamicRange = false;
    viewer.scene.light = new Cesium.DirectionalLight({ direction: new Cesium.Cartesian3(-0.4, -0.6, -1), intensity: 3 });
    viewer.scene.screenSpaceCameraController.enableInputs = false;
    viewer.canvas.tabIndex = -1;
    const frame = Cesium.Transforms.eastNorthUpToFixedFrame(Cesium.Cartesian3.fromDegrees(-84.3, 35.9));
    let active;
    let cancelPending = () => {};
    let generation = 0;
    let orbit = { ...presets.perspective, range: 8 };
    let pointers = new Map();

    function move(patch, label = "Custom view") {
        if (active) {
            orbit = { ...orbit, ...patch };
            orbit.pitch = Math.max(-Math.PI / 2 + 0.01, Math.min(Math.PI / 2 - 0.01, orbit.pitch));
            orbit.range = Math.max(active.radius * 1.15, Math.min(active.radius * 20, orbit.range));
            viewer.camera.lookAt(active.center, new Cesium.HeadingPitchRange(orbit.heading, orbit.pitch, orbit.range));
            viewer.scene.requestRender();
            viewName.textContent = label;
        }
    }
    function reset(name = "perspective") {
        if (active) move({ ...presets[name], range: active.radius * 2.7 }, name.charAt(0).toUpperCase() + name.slice(1));
    }
    async function select(asset) {
        const ticket = ++generation;
        cancelPending();
        report(`Loading ${asset.label}…`);
        viewport.setAttribute("aria-busy", "true");
        updateNavigation();
        try {
            const model = await Cesium.Model.fromGltfAsync({ url: `/models/${asset.file}`, modelMatrix: frame });
            if (ticket !== generation) {
                model.destroy();
            } else {
                await new Promise((ready, reject) => {
                    const timer = setTimeout(() => reject(new Error("Model loading timed out.")), 30000);
                    const removeReady = model.readyEvent.addEventListener(() => { clearTimeout(timer); ready(); });
                    const removeError = model.errorEvent.addEventListener((error) => { clearTimeout(timer); reject(error); });
                    cancelPending = () => {
                        clearTimeout(timer);
                        removeReady();
                        removeError();
                        viewer.scene.primitives.remove(model);
                        reject(new Error("Model selection changed."));
                    };
                    viewer.scene.primitives.add(model);
                    viewer.scene.requestRender();
                });
                if (ticket === generation) {
                    if (active) viewer.scene.primitives.remove(active.model);
                    active = { asset, model, center: Cesium.Cartesian3.clone(model.boundingSphere.center), radius: Math.max(model.boundingSphere.radius, 0.01) };
                    cancelPending = () => {};
                    describe(asset);
                    reset();
                    viewport.dataset.asset = asset.id;
                    viewport.setAttribute("aria-label", `Interactive 3D model: ${asset.label}`);
                    viewport.setAttribute("aria-busy", "false");
                    report(`${asset.label} ready`);
                }
            }
        } catch {
            if (ticket === generation) {
                cancelPending();
                cancelPending = () => {};
                assetPicker.value = active?.asset.id ?? asset.id;
                viewport.setAttribute("aria-busy", "false");
                report(`Could not load ${asset.label}.${active ? ` ${active.asset.label} is still shown.` : " Check the GLB file and refresh to retry."}`, true);
            }
        } finally {
            if (ticket === generation) updateNavigation();
        }
    }
    function zoom(factor) {
        move({ range: orbit.range * factor });
    }
    const search = document.getElementById("asset-search");
    const categories = document.getElementById("categories");
    const clear = document.getElementById("clear-filters");
    const nextModel = document.getElementById("next-model");
    const previousModel = document.getElementById("previous-model");
    const labels = { drone: "Drones", landmarks: "USA landmarks", marker: "Markers", other: "Other", research: "Research & ACORN", risk: "Risk pieces" };

    function adjacentOption(direction) {
        const index = assetPicker.selectedIndex;
        return assetPicker.options[index < 0 ? (direction > 0 ? 0 : assetPicker.options.length - 1) : index + direction];
    }
    function updateNavigation() {
        const loading = viewport.getAttribute("aria-busy") === "true";
        previousModel.disabled = loading || !adjacentOption(-1);
        nextModel.disabled = loading || !adjacentOption(1);
    }
    function browse(direction) {
        const option = adjacentOption(direction);
        if (option) {
            assetPicker.value = option.value;
            void select(assets.find((asset) => asset.id === option.value));
        }
    }
    function filter() {
        const query = search.value.trim().toLocaleLowerCase();
        const matches = assets.filter((asset) => (!categories.value || asset.category === categories.value) && `${asset.label} ${asset.category}`.toLocaleLowerCase().includes(query));
        const selected = assets.some((asset) => asset.id === assetPicker.value) ? assetPicker.value : (active?.asset.id ?? initial.id);
        const folders = [...new Set(matches.map(modelFolder))].sort();
        assetPicker.replaceChildren(...folders.map((folder) => {
            const group = document.createElement("optgroup");
            group.label = folder.charAt(0).toUpperCase() + folder.slice(1);
            group.replaceChildren(...matches.filter((asset) => modelFolder(asset) === folder)
                .sort((left, right) => left.label.localeCompare(right.label))
                .map((asset) => new Option(asset.label, asset.id)));
            return group;
        }));
        assetPicker.value = matches.some((asset) => asset.id === selected) ? selected : "";
        assetPicker.disabled = matches.length === 0;
        document.getElementById("result-count").textContent = `${matches.length} of ${assets.length}`;
        document.getElementById("empty-results").hidden = matches.length > 0;
        clear.hidden = !query && !categories.value;
        updateNavigation();
    }
    categories.replaceChildren(...["", ...new Set(assets.map((asset) => asset.category))].map((category) =>
        new Option(`${labels[category] ?? "All models"} (${assets.filter((asset) => !category || asset.category === category).length})`, category)));
    categories.addEventListener("change", filter);
    search.addEventListener("input", filter);
    clear.addEventListener("click", () => { categories.value = ""; search.value = ""; filter(); search.focus(); });
    filter();
    const theme = document.getElementById("theme-toggle");
    function setTheme(value) {
        document.body.dataset.theme = value;
        theme.setAttribute("aria-pressed", String(value === "dark"));
        theme.setAttribute("aria-label", `Switch to ${value === "dark" ? "light" : "dark"} theme (current: ${value === "dark" ? "Dark" : "Light"})`);
        document.getElementById("theme-label").textContent = value === "dark" ? "Dark" : "Light";
        viewer.scene.backgroundColor = Cesium.Color.fromCssColorString(getComputedStyle(document.body).getPropertyValue("--canvas").trim());
        viewer.scene.requestRender();
    }
    try { setTheme(localStorage.getItem("acorn-model-theme") === "light" ? "light" : "dark"); } catch { setTheme("dark"); }
    theme.addEventListener("click", () => {
        setTheme(document.body.dataset.theme === "dark" ? "light" : "dark");
        try { localStorage.setItem("acorn-model-theme", document.body.dataset.theme); } catch { /* Theme still works when storage is disabled. */ }
    });
    assetPicker.addEventListener("change", () => { void select(assets.find((asset) => asset.id === assetPicker.value)); });
    nextModel.addEventListener("click", () => browse(1));
    previousModel.addEventListener("click", () => browse(-1));
    document.querySelectorAll("[data-view]").forEach((button) => { button.addEventListener("click", () => reset(button.dataset.view)); });
    document.getElementById("zoom-in").addEventListener("click", () => zoom(0.85));
    document.getElementById("zoom-out").addEventListener("click", () => zoom(1.18));
    viewport.addEventListener("contextmenu", (event) => event.preventDefault());
    viewport.addEventListener("pointerdown", (event) => {
        if (event.button === 0) {
            viewport.focus({ preventScroll: true });
            viewport.setPointerCapture(event.pointerId);
            pointers = new Map([...pointers, [event.pointerId, { x: event.clientX, y: event.clientY }]]);
        }
    });
    viewport.addEventListener("pointermove", (event) => {
        const previous = pointers.get(event.pointerId);
        if (previous) {
            const updated = new Map([...pointers, [event.pointerId, { x: event.clientX, y: event.clientY }]]);
            if (pointers.size === 1) move({ heading: orbit.heading - (event.clientX - previous.x) * 0.008, pitch: orbit.pitch + (event.clientY - previous.y) * 0.008 });
            else {
                const distance = (points) => { const [a, b] = [...points.values()]; return Math.hypot(a.x - b.x, a.y - b.y); };
                if (distance(updated) > 0) zoom(distance(pointers) / distance(updated));
            }
            pointers = updated;
        }
    });
    for (const name of ["pointerup", "pointercancel", "lostpointercapture"]) viewport.addEventListener(name, (event) => {
        pointers = new Map([...pointers].filter(([id]) => id !== event.pointerId));
    });
    viewport.addEventListener("wheel", (event) => {
        event.preventDefault();
        zoom(Math.exp(Math.max(-200, Math.min(200, event.deltaY)) * 0.002));
    }, { passive: false });
    viewport.addEventListener("keydown", (event) => {
        const keys = {
            "+": () => zoom(0.85), "=": () => zoom(0.85), "-": () => zoom(1.18),
            ArrowDown: () => move({ pitch: orbit.pitch + 0.12 }), ArrowLeft: () => move({ heading: orbit.heading + 0.15 }),
            ArrowRight: () => move({ heading: orbit.heading - 0.15 }), ArrowUp: () => move({ pitch: orbit.pitch - 0.12 }),
            r: () => reset(), R: () => reset(),
        };
        if (keys[event.key]) { event.preventDefault(); keys[event.key](); }
    });
    window.addEventListener("pagehide", () => { ++generation; cancelPending(); viewer.destroy(); }, { once: true });
    await select(initial);
}
start().catch((error) => {
    document.getElementById("model-name").textContent = "Preview unavailable";
    report(`Unable to open the viewer. ${error.message}`, true);
});
