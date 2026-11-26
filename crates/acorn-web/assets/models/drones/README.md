# Industry and research drone miniatures

Twelve original illustrative GLBs cover surveying, infrastructure inspection, autonomy development and small-robot research. This is a representative selection, not a statistically established worldwide top-12 ranking. Public sources do not establish a full ORNL aircraft inventory or relative usage counts. Only Strelka has an ORNL-specific association in the sources below; inclusion of other platforms does not imply ORNL ownership or endorsement.

Each file has an adjacent editable `.model.json` recipe. Geometry is authored from scratch as a simplified silhouette, with ORNL colors, no vendor logos and no imported manufacturer meshes. These are visualization assets, not engineering CAD, aerodynamic geometry, sensor calibration models or manufacturer-accurate dimensions. Four-rotor layouts, landing gear, camera positions and major wing/body forms distinguish the platforms. Propellers are static. The model and pedestal are separate nodes.

| GLB | Reference and selection context |
|---|---|
| [autel-evo-max-4t.glb](autel-evo-max-4t.glb) | [Autel EVO Max 4T family](https://www.autelrobotics.com/productdetail/evo-max-4t/): compact multi-sensor inspection quadcopter; manufacturer page now describes V2. |
| [bitcraze-crazyflie-2-1.glb](bitcraze-crazyflie-2-1.glb) | [Bitcraze Crazyflie 2.1 family](https://www.bitcraze.io/products/crazyflie-2-1-plus/): small open research quadcopter, exposed board and motor pods; current page covers 2.1+. |
| [dji-matrice-350-rtk.glb](dji-matrice-350-rtk.glb) | [DJI Matrice 350 RTK](https://enterprise.dji.com/matrice-350-rtk): larger payload-carrying industrial quadcopter, skids and twin antennas. |
| [dji-mavic-3-enterprise.glb](dji-mavic-3-enterprise.glb) | [DJI Mavic 3 Enterprise](https://enterprise.dji.com/mavic-3-enterprise): compact folding-arm surveying platform, front camera and RTK module. |
| [dji-phantom-4-rtk.glb](dji-phantom-4-rtk.glb) | [DJI Phantom 4 RTK](https://enterprise.dji.com/phantom-4-rtk): established mapping form, rounded shell, landing skids and RTK mast. |
| [ebee-x.glb](ebee-x.glb) | [eBee X](https://eaglenxt.com/products/drones/ebee-series/ebee-x/): fixed-wing mapping aircraft with swept wings and rear pusher propeller. |
| [freefly-astro.glb](freefly-astro.glb) | [Freefly Astro](https://freeflysystems.com/astro): modular camera/payload quadcopter, twin batteries and open landing gear. |
| [holybro-x500-v2.glb](holybro-x500-v2.glb) | [Holybro X500 V2](https://docs.holybro.com/drone-development-kit/px4-development-kit-x500v2): PX4 development kit with carbon-tube arms, stacked plates and GPS mast. |
| [ornl-strelka.glb](ornl-strelka.glb) | [ORNL vehicle-development factsheet](https://www.ornl.gov/sites/default/files/2023-08/GSHS%20Factsheet_Vehicle_Development.pdf) identifies Strelka as a custom heavy-lift quadcopter for radiological sensing and research. The miniature illustrates this airframe family and a generic instrument package. |
| [parrot-anafi-ai.glb](parrot-anafi-ai.glb) | [Parrot ANAFI Ai](https://www.parrot.com/en/newsroom/parrot-anafi-ai): slender professional quadcopter with distinctive stereo obstacle-sensing “eyes.” |
| [skydio-x10.glb](skydio-x10.glb) | [Skydio X10](https://www.skydio.com/x10): compact inspection/autonomy quadcopter, raised rear arms and multi-sensor front payload. |
| [wingtraone-gen-ii.glb](wingtraone-gen-ii.glb) | [WingtraOne GEN II](https://wingtra.com/wingtraone-gen-ii-drone/): twin-propeller tailsitter survey aircraft, represented in level flight. Wingtra identifies this as its earlier flagship, succeeded by WingtraRAY. |

Sources were consulted November 26, 2026. ORNL's [environmental and security drone research article](https://www.ornl.gov/news/drones-give-birds-eye-view-collecting-environmental-and-security-data) supplies use-case context, rather than a model-by-model fleet list. The manufacturer pages support platform identity and broad form/use, not comparative popularity. No external files or network requests are needed to display these GLBs.

Re-author with `python3 crates/acorn-web/scripts/model-collection.py --category drone`, then run the builder and validator described in the [asset guide](../README.md).
