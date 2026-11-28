# Plain globe markers

Six self-contained GLBs share the DOE laboratory markers’ beveled pin body and metallic Graphite rim. They contain no logo, lettering or pedestal.

| File | Palette finish |
| --- | --- |
| [plain-blue.glb](plain-blue.glb) | Infinity blue |
| [plain-brown.glb](plain-brown.glb) | Burnt umber |
| [plain-green.glb](plain-green.glb) | ORNL Green |
| [plain-magenta.glb](plain-magenta.glb) | Plasma |
| [plain-purple.glb](plain-purple.glb) | Pulsar |
| [plain-teal.glb](plain-teal.glb) | Hydro |

Stable IDs use `marker/plain-{color}`; the viewer groups these files under **Markers**. Units are meters, +Y is up, +Z is front, and the bottom point is at the origin. The 3.37-meter authoring height is illustrative; scale it for globe placement.

Each editable `.model.json` sits beside its GLB. Regenerate with `model-lab-markers.py`, then `build-models.py`, as documented in the [library README](../README.md). Geometry and authoring code are MIT-licensed. These assets remain excluded from crate releases.

[Rendered preview](../previews/marker-overview.png)
