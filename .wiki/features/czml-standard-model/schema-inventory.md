# CZML schema inventory — November 26, 2026

This is the field-level reference for the [CZML standard data model](index.md#execution-phases) phases. The tables were generated from czml-writer wiki pages `CZML/*.md` at `017e7dc1616ae9c893d0b5d160b9f27146f6115c`. Encodings are paraphrased, not copied. Property names and enum constants are reproduced as facts. Everything except the drift noted below was checked against czml-writer `Schema/` at `61cd1d5a890ad955da081250849b99c5fd238921` for the packet property set and the 13 enumerations. CZM-08 replaces this hand check with `tests/fixtures/schema/czml/schema-inventory.json`, generated from the schema, which becomes authoritative.

## Known drift and scope notes

- `Schema/Packet.json` at `61cd1d5a` has 34 properties: 28 core keys and six AGI extension keys. The wiki's `Packet.md` lists 32, including no `agi_groundElevationVolume` (`Schema/Extensions/AGI/GroundElevationVolume.json`). The generated inventory and conformance test use the schema's exact field set.
- `Schema/AlignedAxis.json` uses `unitCartesian`, while `Schema/Examples/Billboard.json` uses `alignedAxis.cartesian`. The model accepts the example's `cartesian` key as a reviewed compatibility field.
- `Schema/Clock.json` omits `interval`, while `Schema/Examples/Clock.json` includes it. The model accepts `Clock.interval` as a reviewed compatibility field.
- `Schema/Examples/Uri.json` shows interval objects under a `uri` key. It is decoded through the `Tileset` wrapper, matching the example's shape.
- The wiki has one page per value type (`*Value.md`). The schema splits each into `Values/<Name>Value.json` and `ValueProperties/<Name>ValueProperty.json`. `ClockRange`, `ClockStep`, `Time`, and `TimeIntervalCollection` become `Values/ClockRangeValue.json`, `Values/ClockStepValue.json`, `Values/TimeValue.json`, and `Values/TimeIntervalCollectionValue.json` (`"type": ["array", "string"]`).
- Required fields use the custom keyword `czmlRequiredForDisplay`, not JSON Schema `required`. They are marked `*` below.
- Every property may also be written as an interval list, and string, number, and boolean properties also have a bare shorthand. The property envelope in the [contract decisions](index.md#contract-decisions) covers this, so the tables below list each value form once.

## Graphics

`*` marks `czmlRequiredForDisplay`: required on the merged entity, optional in any single packet. Every listed property is itself a property type from [Property types](#property-types).

| Type | Properties |
|---|---|
| `Billboard` | `show`, `image`*, `scale`, `pixelOffset`, `eyeOffset`, `horizontalOrigin`, `verticalOrigin`, `heightReference`, `color`, `rotation`, `alignedAxis`, `sizeInMeters`, `width`, `height`, `scaleByDistance`, `translucencyByDistance`, `pixelOffsetScaleByDistance`, `imageSubRegion`, `distanceDisplayCondition`, `disableDepthTestDistance` |
| `Box` | `show`, `dimensions`*, `heightReference`, `fill`, `material`, `outline`, `outlineColor`, `outlineWidth`, `shadows`, `distanceDisplayCondition` |
| `Corridor` | `show`, `positions`*, `width`*, `height`, `heightReference`, `extrudedHeight`, `extrudedHeightReference`, `cornerType`, `granularity`, `fill`, `material`, `outline`, `outlineColor`, `outlineWidth`, `shadows`, `distanceDisplayCondition`, `classificationType`, `zIndex` |
| `Cylinder` | `show`, `length`*, `topRadius`*, `bottomRadius`*, `heightReference`, `fill`, `material`, `outline`, `outlineColor`, `outlineWidth`, `numberOfVerticalLines`, `slices`, `shadows`, `distanceDisplayCondition` |
| `Ellipse` | `show`, `semiMajorAxis`*, `semiMinorAxis`*, `height`, `heightReference`, `extrudedHeight`, `extrudedHeightReference`, `rotation`, `stRotation`, `granularity`, `fill`, `material`, `outline`, `outlineColor`, `outlineWidth`, `numberOfVerticalLines`, `shadows`, `distanceDisplayCondition`, `classificationType`, `zIndex` |
| `Ellipsoid` | `show`, `radii`*, `innerRadii`, `minimumClock`, `maximumClock`, `minimumCone`, `maximumCone`, `heightReference`, `fill`, `material`, `outline`, `outlineColor`, `outlineWidth`, `stackPartitions`, `slicePartitions`, `subdivisions`, `shadows`, `distanceDisplayCondition` |
| `Label` | `show`, `text`, `font`, `style`, `scale`, `showBackground`, `backgroundColor`, `backgroundPadding`, `pixelOffset`, `eyeOffset`, `horizontalOrigin`, `verticalOrigin`, `heightReference`, `fillColor`, `outlineColor`, `outlineWidth`, `translucencyByDistance`, `pixelOffsetScaleByDistance`, `scaleByDistance`, `distanceDisplayCondition`, `disableDepthTestDistance` |
| `Model` | `show`, `gltf`*, `scale`, `minimumPixelSize`, `maximumScale`, `incrementallyLoadTextures`, `runAnimations`, `shadows`, `heightReference`, `silhouetteColor`, `silhouetteSize`, `color`, `colorBlendMode`, `colorBlendAmount`, `distanceDisplayCondition`, `nodeTransformations`, `articulations` |
| `Path` | `show`, `leadTime`, `trailTime`, `width`, `resolution`, `material`, `materialMode`, `distanceDisplayCondition`, `relativeTo` |
| `Point` | `show`, `pixelSize`, `heightReference`, `color`, `outlineColor`, `outlineWidth`, `scaleByDistance`, `translucencyByDistance`, `distanceDisplayCondition`, `disableDepthTestDistance` |
| `Polygon` | `show`, `positions`*, `holes`, `arcType`, `height`, `heightReference`, `extrudedHeight`, `extrudedHeightReference`, `stRotation`, `granularity`, `fill`, `material`, `outline`, `outlineColor`, `outlineWidth`, `perPositionHeight`, `closeTop`, `closeBottom`, `shadows`, `distanceDisplayCondition`, `classificationType`, `zIndex` |
| `Polyline` | `show`, `positions`*, `arcType`, `width`, `granularity`, `material`, `followSurface`, `shadows`, `depthFailMaterial`, `distanceDisplayCondition`, `clampToGround`, `classificationType`, `zIndex` |
| `PolylineVolume` | `show`, `positions`*, `shape`*, `cornerType`, `granularity`, `fill`, `material`, `outline`, `outlineColor`, `outlineWidth`, `shadows`, `distanceDisplayCondition` |
| `Rectangle` | `show`, `coordinates`*, `height`, `heightReference`, `extrudedHeight`, `extrudedHeightReference`, `rotation`, `stRotation`, `granularity`, `fill`, `material`, `outline`, `outlineColor`, `outlineWidth`, `shadows`, `distanceDisplayCondition`, `classificationType`, `zIndex` |
| `Tileset` | `show`, `uri`*, `maximumScreenSpaceError` |
| `Wall` | `show`, `positions`*, `minimumHeights`, `maximumHeights`, `granularity`, `fill`, `material`, `outline`, `outlineColor`, `outlineWidth`, `shadows`, `distanceDisplayCondition` |

## Materials

| Type | Properties |
|---|---|
| `Material` | `solidColor`, `image`, `grid`, `stripe`, `checkerboard` |
| `PolylineMaterial` | `solidColor`, `polylineOutline`, `polylineArrow`, `polylineDash`, `polylineGlow`, `image`, `grid`, `stripe`, `checkerboard` |
| `SolidColorMaterial` | `color` |
| `ImageMaterial` | `image`, `repeat`, `color`, `transparent` |
| `GridMaterial` | `color`, `cellAlpha`, `lineCount`, `lineThickness`, `lineOffset` |
| `StripeMaterial` | `orientation`, `evenColor`, `oddColor`, `offset`, `repeat` |
| `CheckerboardMaterial` | `evenColor`, `oddColor`, `repeat` |
| `PolylineOutlineMaterial` | `color`, `outlineColor`, `outlineWidth` |
| `PolylineArrowMaterial` | `color` |
| `PolylineDashMaterial` | `color`, `gapColor`, `dashLength`, `dashPattern` |
| `PolylineGlowMaterial` | `color`, `glowPower`, `taperPower` |

## Structural types

| Type | Properties or shape |
|---|---|
| `Packet` | `id`, `delete`, `name`, `parent`, `description`→String, `clock`→Clock (document only), `version` (document only), `availability`→TimeIntervalCollection, `properties`→CustomProperties, `position`, `orientation`, `viewFrom`, the 16 graphics keys (lowercase-camel type names, `polylineVolume`), and six `agi_*` keys |
| `Document` | JSON array of `Packet` (schema `Document.json`) |
| `Clock` | `interval`, `currentTime`→Time, `multiplier` (number, default 1.0), `range`→ClockRange (default `LOOP_STOP`), `step`→ClockStep (default `SYSTEM_CLOCK_MULTIPLIER`) |
| `CustomProperties` | map from custom name to `CustomProperty` |
| `NodeTransformations` | map from glTF node name to `NodeTransformation` (`translation`, `rotation`, `scale`) |
| `Articulations` | map from `"<articulation> <stage>"` to `Articulation` |
| `Time` / `TimeIntervalCollection` | ISO 8601 time string / one ISO 8601 interval string or an array of them |

## Property types

Interpolatable types also take the `InterpolatableProperty` base fields (`epoch`, `interpolationAlgorithm`, `interpolationDegree`, `forwardExtrapolationType`, `forwardExtrapolationDuration`, `backwardExtrapolationType`, `backwardExtrapolationDuration`). All of them take `DeletableProperty.delete` and, through the envelope, `interval`.

| Type | Interpolatable | Value keys |
|---|---|---|
| `AlignedAxis` | yes | `unitCartesian`→UnitCartesian3Value, `unitSpherical`→UnitSphericalValue, `reference`→ReferenceValue, `velocityReference`→VelocityReferenceValue |
| `ArcType` | no | `arcType`→ArcTypeValue, `reference`→ReferenceValue |
| `Articulation` | yes | `number`→DoubleValue, `reference`→ReferenceValue |
| `BackgroundPadding` | yes | `cartesian2`→Cartesian2Value, `reference`→ReferenceValue |
| `Boolean` | no | `boolean`→BooleanValue, `reference`→ReferenceValue |
| `BoundingRectangle` | yes | `boundingRectangle`→BoundingRectangleValue, `reference`→ReferenceValue |
| `BoxDimensions` | yes | `cartesian`→Cartesian3Value, `reference`→ReferenceValue |
| `ClassificationType` | no | `classificationType`→ClassificationTypeValue, `reference`→ReferenceValue |
| `Color` | yes | `rgba`→RgbaValue, `rgbaf`→RgbafValue, `reference`→ReferenceValue |
| `ColorBlendMode` | no | `colorBlendMode`→ColorBlendModeValue, `reference`→ReferenceValue |
| `CornerType` | no | `cornerType`→CornerTypeValue, `reference`→ReferenceValue |
| `DistanceDisplayCondition` | yes | `distanceDisplayCondition`→DistanceDisplayConditionValue, `reference`→ReferenceValue |
| `Double` | yes | `number`→DoubleValue, `reference`→ReferenceValue |
| `DoubleList` | no | `array`→DoubleListValue, `references`→ReferenceListValue |
| `EllipsoidRadii` | yes | `cartesian`→Cartesian3Value, `reference`→ReferenceValue |
| `EyeOffset` | yes | `cartesian`→Cartesian3Value, `reference`→ReferenceValue |
| `Font` | no | `font`→FontValue, `reference`→ReferenceValue |
| `HeightReference` | no | `heightReference`→HeightReferenceValue, `reference`→ReferenceValue |
| `HorizontalOrigin` | no | `horizontalOrigin`→HorizontalOriginValue, `reference`→ReferenceValue |
| `Integer` | yes | `number`→IntegerValue, `reference`→ReferenceValue |
| `LabelStyle` | no | `labelStyle`→LabelStyleValue, `reference`→ReferenceValue |
| `LineCount` | yes | `cartesian2`→Cartesian2Value, `reference`→ReferenceValue |
| `LineOffset` | yes | `cartesian2`→Cartesian2Value, `reference`→ReferenceValue |
| `LineThickness` | yes | `cartesian2`→Cartesian2Value, `reference`→ReferenceValue |
| `NearFarScalar` | yes | `nearFarScalar`→NearFarScalarValue, `reference`→ReferenceValue |
| `Orientation` | yes | `unitQuaternion`→UnitQuaternionValue, `reference`→ReferenceValue, `velocityReference`→VelocityReferenceValue |
| `PathMode` | no | `pathMode`→PathModeValue, `reference`→ReferenceValue |
| `PixelOffset` | yes | `cartesian2`→Cartesian2Value, `reference`→ReferenceValue |
| `Position` | yes | `referenceFrame`→string, `cartesian`→Cartesian3Value, `cartographicRadians`→CartographicRadiansValue, `cartographicDegrees`→CartographicDegreesValue, `cartesianVelocity`→Cartesian3VelocityValue, `reference`→ReferenceValue |
| `PositionList` | no | `referenceFrame`→string, `cartesian`→Cartesian3ListValue, `cartographicRadians`→CartographicRadiansListValue, `cartographicDegrees`→CartographicDegreesListValue, `references`→ReferenceListValue |
| `PositionListOfLists` | no | `cartesian`→Cartesian3ListOfListsValue, `cartographicRadians`→CartographicRadiansListOfListsValue, `cartographicDegrees`→CartographicDegreesListOfListsValue, `references`→ReferenceListOfListsValue |
| `RectangleCoordinates` | yes | `wsen`→CartographicRectangleRadiansValue, `wsenDegrees`→CartographicRectangleDegreesValue, `reference`→ReferenceValue |
| `Repeat` | yes | `cartesian2`→Cartesian2Value, `reference`→ReferenceValue |
| `Rotation` | yes | `unitQuaternion`→UnitQuaternionValue, `reference`→ReferenceValue |
| `Scale` | yes | `cartesian`→Cartesian3Value, `reference`→ReferenceValue |
| `ShadowMode` | no | `shadowMode`→ShadowModeValue, `reference`→ReferenceValue |
| `Shape` | no | `cartesian2`→Cartesian2ListValue |
| `String` | no | `string`→StringValue, `reference`→ReferenceValue |
| `StripeOrientation` | no | `stripeOrientation`→StripeOrientationValue, `reference`→ReferenceValue |
| `Translation` | yes | `cartesian`→Cartesian3Value, `reference`→ReferenceValue |
| `Uri` | no | `uri`→UriValue, `reference`→ReferenceValue |
| `VerticalOrigin` | no | `verticalOrigin`→VerticalOriginValue, `reference`→ReferenceValue |
| `ViewFrom` | yes | `cartesian`→Cartesian3Value, `reference`→ReferenceValue |
| `CustomProperty` | yes | `boolean`, `boundingRectangle`, `cartesian`, `cartographicRadians`, `cartographicDegrees`, `cartesian2`, `unitCartesian`, `spherical`, `unitSpherical`, `rgba`, `rgbaf`, `colorBlendMode`, `cornerType`, `heightReference`, `horizontalOrigin`, `labelStyle`, `number`, `nearFarScalar`, `unitQuaternion`, `pathMode`, `shadowMode`, `string`, `stripeOrientation`, `wsen`, `wsenDegrees`, `uri`, `verticalOrigin` |

## Value encodings

| Value type | JSON type | Encoding (N = components) |
|---|---|---|
| `ArcTypeValue` | string | enumeration constant; see [Enumerations](#enumerations) |
| `BooleanValue` | boolean | boolean |
| `BoundingRectangleValue` | array | constant: exactly 4 (`X, Y, Width, Height`); sampled: length multiple of 5 as `[Time, …]` with ISO 8601 or epoch seconds |
| `Cartesian2ListValue` | array | flat multiple of 2: `X, Y`; no time tags |
| `Cartesian2Value` | array | constant: exactly 2 (`X, Y`); sampled: length multiple of 3 as `[Time, …]` with ISO 8601 or epoch seconds |
| `Cartesian3ListOfListsValue` | array | array of flat lists; each inner list is a flat multiple of 3 (`X, Y, Z`) |
| `Cartesian3ListValue` | array | flat multiple of 3: `X, Y, Z`; no time tags |
| `Cartesian3Value` | array | constant: exactly 3 (`X, Y, Z`); sampled: length multiple of 4 as `[Time, …]` with ISO 8601 or epoch seconds |
| `Cartesian3VelocityValue` | array | constant: exactly 6 (`X, Y, Z, dX, dY, dZ`); sampled: length multiple of 7 as `[Time, …]` with ISO 8601 or epoch seconds |
| `CartographicDegreesListOfListsValue` | array | array of flat lists; each inner list is a flat multiple of 3 (`Longitude, Latitude, Height`) |
| `CartographicDegreesListValue` | array | flat multiple of 3: `Longitude, Latitude, Height`; no time tags |
| `CartographicDegreesValue` | array | constant: exactly 3 (`Longitude, Latitude, Height`); sampled: length multiple of 4 as `[Time, …]` with ISO 8601 or epoch seconds |
| `CartographicRadiansListOfListsValue` | array | array of flat lists; each inner list is a flat multiple of 3 (`Longitude, Latitude, Height`) |
| `CartographicRadiansListValue` | array | flat multiple of 3: `Longitude, Latitude, Height`; no time tags |
| `CartographicRadiansValue` | array | constant: exactly 3 (`Longitude, Latitude, Height`); sampled: length multiple of 4 as `[Time, …]` with ISO 8601 or epoch seconds |
| `CartographicRectangleDegreesValue` | array | constant: exactly 4 (`WestLongitude, SouthLatitude, EastLongitude, NorthLatitude`); sampled: length multiple of 5 as `[Time, …]` with ISO 8601 or epoch seconds |
| `CartographicRectangleRadiansValue` | array | constant: exactly 4 (`WestLongitude, SouthLatitude, EastLongitude, NorthLatitude`); sampled: length multiple of 5 as `[Time, …]` with ISO 8601 or epoch seconds |
| `ClassificationTypeValue` | string | enumeration constant; see [Enumerations](#enumerations) |
| `ColorBlendModeValue` | string | enumeration constant; see [Enumerations](#enumerations) |
| `CornerTypeValue` | string | enumeration constant; see [Enumerations](#enumerations) |
| `DistanceDisplayConditionValue` | array | constant: exactly 2 (`NearDistance, FarDistance`); sampled: length multiple of 3 as `[Time, …]` with ISO 8601 or epoch seconds |
| `DoubleListValue` | array | array of numbers |
| `DoubleValue` | number or array | number, or one-element array (constant); sampled `[Time, v, …]`, length multiple of 2 |
| `FontValue` | string | string (CSS font) |
| `HeightReferenceValue` | string | enumeration constant; see [Enumerations](#enumerations) |
| `HorizontalOriginValue` | string | enumeration constant; see [Enumerations](#enumerations) |
| `IntegerValue` | number or array | number, or one-element array (constant); sampled `[Time, v, …]`, length multiple of 2 |
| `LabelStyleValue` | string | enumeration constant; see [Enumerations](#enumerations) |
| `NearFarScalarValue` | array | constant: exactly 4 (`NearDistance, NearValue, FarDistance, FarValue`); sampled: length multiple of 5 as `[Time, …]` with ISO 8601 or epoch seconds |
| `PathModeValue` | string | enumeration constant; see [Enumerations](#enumerations) |
| `ReferenceListOfListsValue` | array | array of arrays of reference strings |
| `ReferenceListValue` | array | array of reference strings |
| `ReferenceValue` | string | `objectId#property.path` |
| `RgbaValue` | array | constant: exactly 4 (`Red, Green, Blue, Alpha`); sampled: length multiple of 5 as `[Time, …]` with ISO 8601 or epoch seconds |
| `RgbafValue` | array | constant: exactly 4 (`Red, Green, Blue, Alpha`); sampled: length multiple of 5 as `[Time, …]` with ISO 8601 or epoch seconds |
| `ShadowModeValue` | string | enumeration constant; see [Enumerations](#enumerations) |
| `SphericalListValue` | array | flat multiple of 3: `Clock, Cone, Magnitude`; no time tags |
| `SphericalValue` | array | constant: exactly 3 (`Clock, Cone, Magnitude`); sampled: length multiple of 4 as `[Time, …]` with ISO 8601 or epoch seconds |
| `StringValue` | string | string |
| `StripeOrientationValue` | string | enumeration constant; see [Enumerations](#enumerations) |
| `UnitCartesian3ListValue` | array | flat multiple of 3: `X, Y, Z`; no time tags |
| `UnitCartesian3Value` | array | constant: exactly 3 (`X, Y, Z`); sampled: length multiple of 4 as `[Time, …]` with ISO 8601 or epoch seconds |
| `UnitQuaternionValue` | array | constant: exactly 4 (`X, Y, Z, W`); sampled: length multiple of 5 as `[Time, …]` with ISO 8601 or epoch seconds |
| `UnitSphericalListValue` | array | flat multiple of 2: `Clock, Cone`; no time tags |
| `UnitSphericalValue` | array | constant: exactly 2 (`Clock, Cone`); sampled: length multiple of 3 as `[Time, …]` with ISO 8601 or epoch seconds |
| `UriValue` | string | URI or data URI |
| `VelocityReferenceValue` | string | reference to a `position` property; value is its normalized velocity |
| `VerticalOriginValue` | string | enumeration constant; see [Enumerations](#enumerations) |

## Enumerations

| Type | Constants |
|---|---|
| `ArcTypeValue` | `NONE`, `GEODESIC`, `RHUMB` |
| `ClassificationTypeValue` | `TERRAIN`, `CESIUM_3D_TILE`, `BOTH` |
| `ClockRange` | `UNBOUNDED`, `CLAMPED`, `LOOP_STOP` |
| `ClockStep` | `TICK_DEPENDENT`, `SYSTEM_CLOCK_MULTIPLIER`, `SYSTEM_CLOCK` |
| `ColorBlendModeValue` | `HIGHLIGHT`, `REPLACE`, `MIX` |
| `CornerTypeValue` | `ROUNDED`, `MITERED`, `BEVELED` |
| `HeightReferenceValue` | `NONE`, `CLAMP_TO_GROUND`, `RELATIVE_TO_GROUND` |
| `HorizontalOriginValue` | `LEFT`, `CENTER`, `RIGHT` |
| `LabelStyleValue` | `FILL`, `OUTLINE`, `FILL_AND_OUTLINE` |
| `PathModeValue` | `WHOLE`, `PORTIONS` |
| `ShadowModeValue` | `DISABLED`, `ENABLED`, `CAST_ONLY`, `RECEIVE_ONLY` |
| `StripeOrientationValue` | `HORIZONTAL`, `VERTICAL` |
| `VerticalOriginValue` | `BASELINE`, `BOTTOM`, `CENTER`, `TOP` |
| `InterpolationAlgorithm` (InterpolatableProperty) | `LINEAR` (default), `LAGRANGE`, `HERMITE` |
| `ExtrapolationType` (InterpolatableProperty) | `NONE` (default), `HOLD`, `EXTRAPOLATE` |
| `ReferenceFrame` (Position, PositionList) | `FIXED` (default), `INERTIAL` |

## AGI extensions (CZM-10)

| Type | Properties |
|---|---|
| `ConicSensor` | `show`, `innerHalfAngle`, `outerHalfAngle`, `minimumClockAngle`, `maximumClockAngle`, `minimumRadius`, `maximumRadius`, `radius`, `showIntersection`, `intersectionColor`, `intersectionWidth`, `showLateralSurfaces`, `lateralSurfaceMaterial`, `showEllipsoidSurfaces`, `ellipsoidSurfaceMaterial`, `showEllipsoidHorizonSurfaces`, `ellipsoidHorizonSurfaceMaterial`, `showDomeSurfaces`, `domeSurfaceMaterial`, `portionToDisplay`, `environmentConstraint`, `showEnvironmentOcclusion`, `environmentOcclusionMaterial`, `showEnvironmentIntersection`, `environmentIntersectionColor`, `environmentIntersectionWidth`, `showThroughEllipsoid`, `showViewshed`, `viewshedVisibleColor`, `viewshedOccludedColor`, `viewshedResolution` |
| `CustomPatternSensor` | `show`, `directions`*, `minimumRadius`, `maximumRadius`, `radius`, `showIntersection`, `intersectionColor`, `intersectionWidth`, `showLateralSurfaces`, `lateralSurfaceMaterial`, `showEllipsoidSurfaces`, `ellipsoidSurfaceMaterial`, `showEllipsoidHorizonSurfaces`, `ellipsoidHorizonSurfaceMaterial`, `showDomeSurfaces`, `domeSurfaceMaterial`, `portionToDisplay`, `environmentConstraint`, `showEnvironmentOcclusion`, `environmentOcclusionMaterial`, `showEnvironmentIntersection`, `environmentIntersectionColor`, `environmentIntersectionWidth`, `showThroughEllipsoid`, `showViewshed`, `viewshedVisibleColor`, `viewshedOccludedColor`, `viewshedResolution` |
| `RectangularSensor` | `show`, `xHalfAngle`, `yHalfAngle`, `minimumRadius`, `maximumRadius`, `radius`, `showIntersection`, `intersectionColor`, `intersectionWidth`, `showLateralSurfaces`, `lateralSurfaceMaterial`, `showEllipsoidSurfaces`, `ellipsoidSurfaceMaterial`, `showEllipsoidHorizonSurfaces`, `ellipsoidHorizonSurfaceMaterial`, `showDomeSurfaces`, `domeSurfaceMaterial`, `portionToDisplay`, `environmentConstraint`, `showEnvironmentOcclusion`, `environmentOcclusionMaterial`, `showEnvironmentIntersection`, `environmentIntersectionColor`, `environmentIntersectionWidth`, `showThroughEllipsoid`, `showViewshed`, `viewshedVisibleColor`, `viewshedOccludedColor`, `viewshedResolution` |
| `Fan` | `show`, `directions`*, `radius`, `perDirectionRadius`, `material`, `fill`, `outline`, `outlineColor`, `outlineWidth`, `numberOfRings` |
| `Vector` | `show`, `color`, `direction`*, `length`, `minimumLengthInPixels` |
| `Direction` | `spherical`, `unitSpherical`, `cartesian`, `unitCartesian`, `reference` |
| `DirectionList` | `spherical`, `unitSpherical`, `cartesian`, `unitCartesian` |
| `SensorVolumePortionToDisplay` | `portionToDisplay`, `reference` |

