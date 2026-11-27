//! CZML constants pinned to the czml-writer value schemas.
use super::macros::czml_enum;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
czml_enum! {
    /// Arc construction mode.
    pub enum ArcTypeValue {
        /// No connecting arc.
        None,
        /// Geodesic arc.
        Geodesic,
        /// Rhumb-line arc.
        Rhumb,
    }
}
czml_enum! {
    /// Classification target for geometry.
    pub enum ClassificationTypeValue {
        /// Terrain only.
        Terrain,
        /// Cesium 3D Tiles only.
        #[serde(rename = "CESIUM_3D_TILE")]
        Cesium3dTile,
        /// Terrain and 3D Tiles.
        Both,
    }
}
czml_enum! {
    /// Behavior at the end of a clock interval.
    pub enum ClockRange {
        /// Continue beyond the interval.
        Unbounded,
        /// Stop at the interval boundary.
        Clamped,
        /// Loop to the interval start.
        LoopStop,
    }
}
czml_enum! {
    /// Clock stepping source.
    pub enum ClockStep {
        /// Advance by the elapsed tick time.
        TickDependent,
        /// Advance by wall time multiplied by the multiplier.
        SystemClockMultiplier,
        /// Follow the system clock.
        SystemClock,
    }
}
czml_enum! {
    /// Model color blending mode.
    pub enum ColorBlendModeValue {
        /// Highlight the model's original color.
        Highlight,
        /// Replace the original color.
        Replace,
        /// Mix the authored and original colors.
        Mix,
    }
}
czml_enum! {
    /// Corridor corner geometry.
    pub enum CornerTypeValue {
        /// Rounded corner.
        Rounded,
        /// Mitered corner.
        Mitered,
        /// Beveled corner.
        Beveled,
    }
}
czml_enum! {
    #[derive(Default)]
    /// Behavior outside the range of sampled values.
    pub enum ExtrapolationType {
        /// No extrapolation.
        #[default]
        None,
        /// Hold the endpoint value.
        Hold,
        /// Extrapolate beyond the endpoint.
        Extrapolate,
    }
}
czml_enum! {
    /// Height positioning relative to terrain.
    pub enum HeightReferenceValue {
        /// Absolute height.
        None,
        /// Clamp to terrain.
        ClampToGround,
        /// Height above terrain.
        RelativeToGround,
    }
}
czml_enum! {
    /// Horizontal origin of a label or billboard.
    pub enum HorizontalOriginValue {
        /// Left edge.
        Left,
        /// Center.
        Center,
        /// Right edge.
        Right,
    }
}
czml_enum! {
    #[derive(Default)]
    /// Interpolation method for sampled values.
    pub enum InterpolationAlgorithm {
        /// Linear interpolation.
        #[default]
        Linear,
        /// Lagrange polynomial interpolation.
        Lagrange,
        /// Hermite interpolation.
        Hermite,
    }
}
czml_enum! {
    /// Label fill and outline style.
    pub enum LabelStyleValue {
        /// Fill only.
        Fill,
        /// Outline only.
        Outline,
        /// Fill and outline.
        FillAndOutline,
    }
}
czml_enum! {
    /// Path display mode.
    pub enum PathModeValue {
        /// Display the whole path.
        Whole,
        /// Display path portions.
        Portions,
    }
}
czml_enum! {
    #[derive(Default)]
    /// Coordinate frame for positions and position lists.
    pub enum ReferenceFrame {
        /// Earth-fixed frame.
        #[default]
        Fixed,
        /// Inertial frame.
        Inertial,
    }
}
czml_enum! {
    /// Shadow casting and receiving mode.
    pub enum ShadowModeValue {
        /// Disable shadows.
        Disabled,
        /// Cast and receive shadows.
        Enabled,
        /// Cast only.
        CastOnly,
        /// Receive only.
        ReceiveOnly,
    }
}
czml_enum! {
    /// Stripe material orientation.
    pub enum StripeOrientationValue {
        /// Horizontal stripes.
        Horizontal,
        /// Vertical stripes.
        Vertical,
    }
}
czml_enum! {
    /// Vertical origin of a label or billboard.
    pub enum VerticalOriginValue {
        /// Text baseline.
        Baseline,
        /// Bottom edge.
        Bottom,
        /// Center.
        Center,
        /// Top edge.
        Top,
    }
}
