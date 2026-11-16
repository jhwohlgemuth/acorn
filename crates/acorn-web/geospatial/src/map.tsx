import { Cartesian3, Color, createOsmBuildingsAsync, EllipsoidTerrainProvider, GeographicTilingScheme, ImageryLayer, Ion, OpenStreetMapImageryProvider, Terrain, UrlTemplateImageryProvider, Viewer as CesiumViewer } from "cesium";
import { Component, useEffect, useMemo, useRef, useState } from "react";
import { CameraFlyTo, type CesiumComponentRef, Entity, Scene, Viewer } from "resium";
import type { State } from "./bridge";
import { configureHome } from "./home";

export class MapBoundary extends Component<{ children: React.ReactNode; onError: (message: string) => void }, { failed: boolean }> {
    state = { failed: false };
    static getDerivedStateFromError() { return { failed: true }; }
    componentDidCatch() { this.props.onError("The map is unavailable. All organization locations remain in the table."); }
    render() { return this.state.failed ? null : this.props.children; }
}
const layers = [
    { id: "black-marble", label: "Black Marble", detail: "Offline · NASA night imagery" },
    { id: "natural-earth", label: "Natural Earth II", detail: "Offline · physical map" },
    { id: "blue-marble", label: "NASA Blue Marble", detail: "Online · NASA GIBS" },
    { id: "openstreetmap", label: "OpenStreetMap", detail: "Online · roads and labels" },
] as const;
type LayerId = typeof layers[number]["id"];
function createImagery(id: LayerId, base: string, onError: (message: string) => void) {
    const provider = id === "black-marble" ? new UrlTemplateImageryProvider({
        url: new URL("Assets/Textures/BlackMarble/", base).href + "{z}/{y}/{x}.png",
        maximumLevel: 3,
        credit: "NASA Earth Observatory / NOAA NGDC · Black Marble (2016), NASA GIBS",
    }) : id === "natural-earth" ? new UrlTemplateImageryProvider({
        url: new URL("Assets/Textures/NaturalEarthII/", base).href + "{z}/{x}/{reverseY}.jpg",
        tilingScheme: new GeographicTilingScheme(),
        maximumLevel: 5,
        credit: "Natural Earth II",
    }) : id === "blue-marble" ? new UrlTemplateImageryProvider({
        url: "https://gibs.earthdata.nasa.gov/wmts/epsg3857/best/BlueMarble_NextGeneration/default/GoogleMapsCompatible_Level8/{z}/{y}/{x}.jpeg",
        maximumLevel: 8,
        credit: "NASA Global Imagery Browse Services (GIBS)",
    }) : new OpenStreetMapImageryProvider({
        url: "https://tile.openstreetmap.org/",
        credit: "© OpenStreetMap contributors",
    });
    const layer = new ImageryLayer(provider);
    layer.errorEvent.addEventListener(() => onError(`${layers.find((item) => item.id === id)?.label ?? "Map"} imagery could not load. Choose another layer or use the organization table.`));
    return layer;
}
export function MapView({ state, base, select, error }: { state: State; base: string; select: (id: string) => void; error: (message: string) => void }) {
    const [layerId, setLayerId] = useState<LayerId>("black-marble");
    const [ionToken, setIonToken] = useState("");
    const [ionEnabled, setIonEnabled] = useState(false);
    const [buildingsEnabled, setBuildingsEnabled] = useState(false);
    const viewerRef = useRef<CesiumComponentRef<CesiumViewer>>(null);
    const imagery = useMemo(() => createImagery(layerId, base, error), [base, layerId]);
    const terrain = useMemo(() => new EllipsoidTerrainProvider(), []);
    const ionTerrain = useMemo(() => {
        if (!ionEnabled) return undefined;
        Ion.defaultAccessToken = ionToken.trim();
        const value = Terrain.fromWorldTerrain({ requestVertexNormals: true, requestWaterMask: true });
        value.errorEvent.addEventListener(() => error("Cesium World Terrain could not load. Check the ion token or disable terrain."));
        return value;
    }, [ionEnabled, ionToken]);
    useEffect(() => {
        if (!buildingsEnabled || !viewerRef.current?.cesiumElement) return;
        Ion.defaultAccessToken = ionToken.trim();
        const viewer = viewerRef.current.cesiumElement;
        let cancelled = false;
        let buildings: Awaited<ReturnType<typeof createOsmBuildingsAsync>> | undefined;
        createOsmBuildingsAsync().then((tileset) => {
            if (cancelled) tileset.destroy();
            else buildings = viewer.scene.primitives.add(tileset);
        }).catch(() => error("Cesium OSM Buildings could not load. Check the ion token or disable buildings."));
        return () => {
            cancelled = true;
            if (buildings && !buildings.isDestroyed()) viewer.scene.primitives.remove(buildings);
        };
    }, [buildingsEnabled, ionToken]);
    const chosen = state.markers.find((marker) => marker.id === state.selected)?.location;
    const selectedLayer = layers.find((layer) => layer.id === layerId) ?? layers[0];
    return <><Viewer ref={viewerRef} full extend={configureHome} animation={false} timeline={false} baseLayer={imagery} terrain={ionTerrain} terrainProvider={ionTerrain ? undefined : terrain}
        baseLayerPicker={false} geocoder={false} homeButton={true} sceneModePicker={true} navigationHelpButton={false}
        infoBox={false} selectionIndicator={true} fullscreenButton={true} fullscreenElement="geospatial-map" requestRenderMode={true}
        onSelectedEntityChange={(entity) => { if (entity) select(entity.id); }}
        >
        <Scene onRenderError={() => error("WebGL rendering stopped. Use the organization table.")} />
        {state.markers.filter((marker) => marker.location).map((marker) => <Entity key={marker.id} id={marker.id}
            name={marker.name} properties={{ ror: marker.id, country: marker.location!.country, latitude: marker.location!.latitude, longitude: marker.location!.longitude, references: marker.references }} position={Cartesian3.fromDegrees(marker.location!.longitude, marker.location!.latitude)}
            point={{ pixelSize: marker.id === state.selected ? 14 : 9, color: Color.fromCssColorString("#39ab68"), outlineColor: Color.WHITE, outlineWidth: 2 }} />)}
        {chosen && <CameraFlyTo duration={0} destination={Cartesian3.fromDegrees(chosen.longitude, chosen.latitude, 3000000)} />}
    </Viewer>
    <details className="map-layer-widget">
        <summary title={`Map layers: ${selectedLayer.label}`} aria-label={`Map layers: ${selectedLayer.label}`}><svg viewBox="0 0 24 24" aria-hidden="true"><path d="m12 3 9 5-9 5-9-5 9-5Z"/><path d="m3 12 9 5 9-5M3 16l9 5 9-5"/></svg></summary>
        <div className="map-layer-panel">
            <strong>{selectedLayer.label}</strong>
            <fieldset className="map-basemap"><legend>Basemap</legend>
                {layers.map((layer) => <label className={layer.id === layerId ? "map-basemap-option selected" : "map-basemap-option"} key={layer.id}>
                    <input type="radio" name="basemap" value={layer.id} checked={layer.id === layerId} onChange={() => setLayerId(layer.id)} />
                    <span><strong>{layer.label}</strong><small>{layer.detail}</small></span>
                </label>)}
            </fieldset>
            <details className="map-ion-options">
                <summary>3D layers <span>Cesium ion</span></summary>
                <div>
                    <label className="map-token"><span>Token</span><input type="password" value={ionToken} autoComplete="off" onChange={(event) => { setIonToken(event.target.value); setIonEnabled(false); setBuildingsEnabled(false); }} placeholder="This tab only" /></label>
                    <label><input type="checkbox" checked={ionEnabled} disabled={!ionToken.trim()} onChange={(event) => setIonEnabled(event.target.checked)} /><span>World Terrain</span></label>
                    <label><input type="checkbox" checked={buildingsEnabled} disabled={!ionToken.trim()} onChange={(event) => setBuildingsEnabled(event.target.checked)} /><span>OSM Buildings</span></label>
                    <small>Online · Cesium ion plan applies</small>
                </div>
            </details>
        </div>
    </details></>;
}
