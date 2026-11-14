import { Cartesian3, type Viewer } from "cesium";

// ORNL's location in the bundled ROR 2.15 index (https://ror.org/01qz5mb56).
export const ornlHome = { latitude: 36.01036, longitude: -84.26964, height: 8000000 };
export function configureHome(viewer: Viewer) {
    const view = {
        destination: Cartesian3.fromDegrees(ornlHome.longitude, ornlHome.latitude, ornlHome.height),
        orientation: { heading: 0, pitch: -Math.PI / 2, roll: 0 },
    };
    viewer.camera.setView(view);
    viewer.homeButton.viewModel.tooltip = "Center on ORNL";
    viewer.homeButton.viewModel.command.beforeExecute.addEventListener((event) => {
        event.cancel = true;
        viewer.camera.flyTo({ ...view, duration: window.matchMedia("(prefers-reduced-motion: reduce)").matches ? 0 : 1 });
    });
}
