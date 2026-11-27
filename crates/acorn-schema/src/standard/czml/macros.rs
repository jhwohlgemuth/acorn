//! Shared CZML declaration and validation macros.

macro_rules! czml_enum {
    ($(#[$type_meta:meta])* pub enum $name:ident { $($(#[$variant_meta:meta])* $variant:ident),+ $(,)? }) => {
        $(#[$type_meta])*
        #[derive(Clone, Copy, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
        #[serde(rename_all = "SCREAMING_SNAKE_CASE")]
        pub enum $name {
            $($(#[$variant_meta])* $variant),+
        }
    };
}
pub(super) use czml_enum;

#[allow(unused_macros, reason = "CZM-03B will instantiate the property definitions")]
macro_rules! czml_property {
    (
        $(#[$meta:meta])* $vis:vis struct $name:ident : $scalar:ty, $allowed:expr, deletable {
            $( $(#[$field_meta:meta])* $field:ident: $field_type:ty, )+
        }
    ) => {
        czml_property!(@emit [$(#[$meta])*] $vis $name : $scalar, $allowed, [
            /// Whether this interval deletes the property.
            pub delete: Option<bool>,
        ] [] { $( $(#[$field_meta])* $field: $field_type, )+ });
    };
    (
        $(#[$meta:meta])* $vis:vis struct $name:ident : $scalar:ty, $allowed:expr, interpolatable {
            $( $(#[$field_meta:meta])* $field:ident: $field_type:ty, )+
        }
    ) => {
        czml_property!(@emit [$(#[$meta])*] $vis $name : $scalar, $allowed, [
            /// Type of backward extrapolation beyond samples.
            pub backward_extrapolation_type: Option<$crate::standard::czml::ExtrapolationType>,
            /// Backward extrapolation duration in seconds.
            pub backward_extrapolation_duration: Option<f64>,
            /// Whether this interval deletes the property.
            pub delete: Option<bool>,
            /// Epoch for numeric sample offsets.
            pub epoch: Option<$crate::standard::czml::Time>,
            /// Forward extrapolation duration in seconds.
            pub forward_extrapolation_duration: Option<f64>,
            /// Type of forward extrapolation beyond samples.
            pub forward_extrapolation_type: Option<$crate::standard::czml::ExtrapolationType>,
            /// Interpolation algorithm for sampled values.
            pub interpolation_algorithm: Option<$crate::standard::czml::InterpolationAlgorithm>,
            /// Interpolation polynomial degree.
            pub interpolation_degree: Option<u32>,
        ] [] { $( $(#[$field_meta])* $field: $field_type, )+ });
    };
    (
        $(#[$meta:meta])* $vis:vis struct $name:ident : $scalar:ty, $allowed:expr, deletable,
        metadata { $( $(#[$setting_meta:meta])* $setting:ident: $setting_type:ty, )+ }
        values { $( $(#[$field_meta:meta])* $field:ident: $field_type:ty, )+ }
    ) => {
        czml_property!(@emit [$(#[$meta])*] $vis $name : $scalar, $allowed, [
            /// Whether this interval deletes the property.
            pub delete: Option<bool>,
        ] [$( $(#[$setting_meta])* pub $setting: Option<$setting_type>, )+] {
            $( $(#[$field_meta])* $field: $field_type, )+
        });
    };
    (
        $(#[$meta:meta])* $vis:vis struct $name:ident : $scalar:ty, $allowed:expr, interpolatable,
        metadata { $( $(#[$setting_meta:meta])* $setting:ident: $setting_type:ty, )+ }
        values { $( $(#[$field_meta:meta])* $field:ident: $field_type:ty, )+ }
    ) => {
        czml_property!(@emit [$(#[$meta])*] $vis $name : $scalar, $allowed, [
            /// Type of backward extrapolation beyond samples.
            pub backward_extrapolation_type: Option<$crate::standard::czml::ExtrapolationType>,
            /// Backward extrapolation duration in seconds.
            pub backward_extrapolation_duration: Option<f64>,
            /// Whether this interval deletes the property.
            pub delete: Option<bool>,
            /// Epoch for numeric sample offsets.
            pub epoch: Option<$crate::standard::czml::Time>,
            /// Forward extrapolation duration in seconds.
            pub forward_extrapolation_duration: Option<f64>,
            /// Type of forward extrapolation beyond samples.
            pub forward_extrapolation_type: Option<$crate::standard::czml::ExtrapolationType>,
            /// Interpolation algorithm for sampled values.
            pub interpolation_algorithm: Option<$crate::standard::czml::InterpolationAlgorithm>,
            /// Interpolation polynomial degree.
            pub interpolation_degree: Option<u32>,
        ] [$( $(#[$setting_meta])* pub $setting: Option<$setting_type>, )+] {
            $( $(#[$field_meta])* $field: $field_type, )+
        });
    };
    (@emit [$($meta:tt)*] $vis:vis $name:ident : $scalar:ty, $allowed:expr, [$($base:tt)*] [$($settings:tt)*] {
        $( $(#[$field_meta:meta])* $field:ident: $field_type:ty, )+
    }) => {
        $($meta)*
        #[serde_with::skip_serializing_none]
        #[derive(Clone, Debug, serde::Deserialize, schemars::JsonSchema, PartialEq, serde::Serialize)]
        #[serde(deny_unknown_fields, rename_all = "camelCase")]
        $vis struct $name {
            /// Interval in which this object applies.
            pub interval: Option<$crate::standard::czml::TimeInterval>,
            $($base)*
            $($settings)*
            $( $(#[$field_meta])* pub $field: Option<$field_type>, )+
        }
        impl $crate::standard::czml::property::PropertyForm for $name {
            type Scalar = $scalar;
            const ALLOW_SCALAR: bool = $allowed;
        }
        impl $crate::validation::Validate for $name {
            fn validate(&self) -> Result<(), $crate::validation::ValidationReport> {
                let count = [$(self.$field.is_some()),+].into_iter().filter(|present| *present).count();
                let report = $crate::validation::report_of(self.interval.as_ref().map($crate::validation::Validate::validate).transpose().map(|_| ()));
                let report = match count <= 1 {
                    | true => report,
                    | false => report.merge("", $crate::validation::ValidationReport::from_error(
                        "", acorn_core::validation::ValidationError::new("czml_property_value_forms")
                            .with_message("At most one CZML value form is allowed"),
                    )),
                };
                $(let report = report.merge(stringify!($field), $crate::validation::report_of(
                    $crate::standard::czml::validate::ValidateSamples::validate_samples(&self.$field)
                ));)+
                report.finish()
            }
        }
    };
}
pub(super) use czml_property;

macro_rules! no_samples {
    ($($kind:ty),+ $(,)?) => {
        $(impl ValidateSamples for $kind {
            fn validate_samples(&self) -> Result<(), ValidationReport> {
                Ok(())
            }
        })+
    };
}
pub(super) use no_samples;
