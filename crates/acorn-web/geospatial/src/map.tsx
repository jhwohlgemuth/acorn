import { Cartesian3, Color, EllipsoidTerrainProvider, ImageryLayer, TileMapServiceImageryProvider } from "cesium";
import { Component, useMemo } from "react";
import { CameraFlyTo, Entity, Scene, Viewer } from "resium";
import type { State } from "./bridge";

export class MapBoundary extends Component<{ children: React.ReactNode; onError: (message: string) => void }, { failed: boolean }> {
    state = { failed: false };
    static getDerivedStateFromError() { return { failed: true }; }
    componentDidCatch() { this.props.onError("The map is unavailable. All organization locations remain in the table."); }
    render() { return this.state.failed ? null : this.props.children; }
}
export function MapView({ state, base, select, error }: { state: State; base: string; select: (id: string) => void; error: (message: string) => void }) {
    const imagery = useMemo(() => {
        const layer = ImageryLayer.fromProviderAsync(TileMapServiceImageryProvider.fromUrl(new URL("Assets/Textures/NaturalEarthII", base).href));
        layer.errorEvent.addEventListener(() => error("Local map imagery could not load. Use the organization table."));
        return layer;
    }, [base]);
    const terrain = useMemo(() => new EllipsoidTerrainProvider(), []);
    const chosen = state.markers.find((marker) => marker.id === state.selected)?.location;
    return <Viewer full animation={false} timeline={false} baseLayer={imagery} terrainProvider={terrain}
        baseLayerPicker={false} geocoder={false} homeButton={true} sceneModePicker={false} navigationHelpButton={false}
        infoBox={false} selectionIndicator={true} fullscreenButton={false} requestRenderMode={true}
        onSelectedEntityChange={(entity) => { if (entity) select(entity.id); }}
        >
        <Scene onRenderError={() => error("WebGL rendering stopped. Use the organization table.")} />
        {state.markers.filter((marker) => marker.location).map((marker) => <Entity key={marker.id} id={marker.id}
            name={marker.name} position={Cartesian3.fromDegrees(marker.location!.longitude, marker.location!.latitude)}
            point={{ pixelSize: marker.id === state.selected ? 14 : 9, color: marker.id === state.selected ? Color.YELLOW : Color.fromCssColorString("#39ab68"), outlineColor: Color.WHITE, outlineWidth: 2 }} />)}
        {chosen && <CameraFlyTo duration={0} destination={Cartesian3.fromDegrees(chosen.longitude, chosen.latitude, 3000000)} />}
    </Viewer>;
}
