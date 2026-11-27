//! Validation of typed CZML samples and document-local rules.
use super::macros::no_samples;
use super::{
    ArcTypeValue, ClassificationTypeValue, Clock, ClockRange, ClockStep, ColorBlendModeValue, CornerTypeValue, Document, ExtrapolationType,
    FixedValue, FlatList, HeightReferenceValue, HorizontalOriginValue, IntegerArrayValue, InterpolationAlgorithm, LabelStyleValue, ListOfLists,
    Packet, PathModeValue, PrimitiveValue, Reference, ReferenceFrame, SampleTime, ScalarValue, ShadowModeValue, StripeOrientationValue,
    VelocityReference, VerticalOriginValue,
};
use crate::validation::{report_of, Validate, ValidationReport};
use acorn_core::prelude::alloc::{format, String, Vec};
use acorn_core::validation::ValidationError;

pub(super) trait ValidateSamples {
    fn validate_samples(&self) -> Result<(), ValidationReport>;
}

no_samples!(
    ArcTypeValue,
    bool,
    ClassificationTypeValue,
    ClockRange,
    ClockStep,
    ColorBlendModeValue,
    CornerTypeValue,
    ExtrapolationType,
    f64,
    HeightReferenceValue,
    HorizontalOriginValue,
    i64,
    InterpolationAlgorithm,
    LabelStyleValue,
    PathModeValue,
    PrimitiveValue,
    ReferenceFrame,
    ShadowModeValue,
    String,
    StripeOrientationValue,
    VerticalOriginValue,
);
impl Validate for Clock {
    fn validate(&self) -> Result<(), ValidationReport> {
        let report = ValidationReport::new()
            .merge(
                "interval",
                report_of(self.interval.as_ref().map(Validate::validate).transpose().map(|_| ())),
            )
            .merge(
                "currentTime",
                report_of(self.current_time.as_ref().map(Validate::validate).transpose().map(|_| ())),
            );
        let report = match self.multiplier.is_some_and(|value| !value.is_finite()) {
            | true => report.merge(
                "multiplier",
                ValidationReport::from_error(
                    "",
                    ValidationError::new("czml_clock_multiplier").with_message("Clock multiplier must be finite"),
                ),
            ),
            | false => report,
        };
        report.finish()
    }
}
impl Validate for Document {
    fn validate(&self) -> Result<(), ValidationReport> {
        let report = report_of(self.0.validate());
        let report = match self.0.first() {
            | Some(packet) if packet.id.as_deref() == Some("document") => report,
            | _ => report.merge(
                "[0].id",
                ValidationReport::from_error(
                    "",
                    ValidationError::new("czml_document_first").with_message("The first packet must be the document packet"),
                ),
            ),
        };
        let report = match self.0.first().and_then(|packet| packet.version.as_deref()) {
            | Some(version) if valid_version(version) => report,
            | _ => report.merge(
                "[0].version",
                ValidationReport::from_error(
                    "",
                    ValidationError::new("czml_version").with_message("The document packet requires CZML version 1.<minor>"),
                ),
            ),
        };
        report.finish()
    }
}
impl<const N: usize> ValidateSamples for FixedValue<N> {
    fn validate_samples(&self) -> Result<(), ValidationReport> {
        match self {
            | Self::Constant(_) => Ok(()),
            | Self::Samples(samples) => sample_order(samples),
        }
    }
}
impl<const N: usize> ValidateSamples for FlatList<N> {
    fn validate_samples(&self) -> Result<(), ValidationReport> {
        Ok(())
    }
}
impl<const N: usize, const MIN: i64, const MAX: i64> ValidateSamples for IntegerArrayValue<N, MIN, MAX> {
    fn validate_samples(&self) -> Result<(), ValidationReport> {
        match self {
            | Self::Constant(_) => Ok(()),
            | Self::Samples(samples) => sample_order(samples),
        }
    }
}
impl<const N: usize> ValidateSamples for ListOfLists<N> {
    fn validate_samples(&self) -> Result<(), ValidationReport> {
        Ok(())
    }
}
impl<T: ValidateSamples> ValidateSamples for Option<T> {
    fn validate_samples(&self) -> Result<(), ValidationReport> {
        self.as_ref().map(ValidateSamples::validate_samples).transpose().map(|_| ())
    }
}
impl Validate for Packet {
    fn validate(&self) -> Result<(), ValidationReport> {
        let report = ValidationReport::new();
        let report = report.merge(
            "availability",
            report_of(self.availability.as_ref().map(Validate::validate).transpose().map(|_| ())),
        );
        let report = report.merge(
            "billboard",
            report_of(self.billboard.as_ref().map(Validate::validate).transpose().map(|_| ())),
        );
        let report = report.merge("box", report_of(self.r#box.as_ref().map(Validate::validate).transpose().map(|_| ())));
        let report = report.merge("clock", report_of(self.clock.as_ref().map(Validate::validate).transpose().map(|_| ())));
        let report = report.merge(
            "corridor",
            report_of(self.corridor.as_ref().map(Validate::validate).transpose().map(|_| ())),
        );
        let report = report.merge(
            "cylinder",
            report_of(self.cylinder.as_ref().map(Validate::validate).transpose().map(|_| ())),
        );
        let report = report.merge(
            "description",
            report_of(self.description.as_ref().map(Validate::validate).transpose().map(|_| ())),
        );
        let report = report.merge(
            "ellipse",
            report_of(self.ellipse.as_ref().map(Validate::validate).transpose().map(|_| ())),
        );
        let report = report.merge(
            "ellipsoid",
            report_of(self.ellipsoid.as_ref().map(Validate::validate).transpose().map(|_| ())),
        );
        let report = report.merge("label", report_of(self.label.as_ref().map(Validate::validate).transpose().map(|_| ())));
        let report = report.merge("model", report_of(self.model.as_ref().map(Validate::validate).transpose().map(|_| ())));
        let report = report.merge(
            "orientation",
            report_of(self.orientation.as_ref().map(Validate::validate).transpose().map(|_| ())),
        );
        let report = report.merge("path", report_of(self.path.as_ref().map(Validate::validate).transpose().map(|_| ())));
        let report = report.merge("point", report_of(self.point.as_ref().map(Validate::validate).transpose().map(|_| ())));
        let report = report.merge(
            "polygon",
            report_of(self.polygon.as_ref().map(Validate::validate).transpose().map(|_| ())),
        );
        let report = report.merge(
            "polyline",
            report_of(self.polyline.as_ref().map(Validate::validate).transpose().map(|_| ())),
        );
        let report = report.merge(
            "polyline_volume",
            report_of(self.polyline_volume.as_ref().map(Validate::validate).transpose().map(|_| ())),
        );
        let report = report.merge(
            "position",
            report_of(self.position.as_ref().map(Validate::validate).transpose().map(|_| ())),
        );
        let report = report.merge(
            "properties",
            report_of(self.properties.as_ref().map(Validate::validate).transpose().map(|_| ())),
        );
        let report = report.merge(
            "rectangle",
            report_of(self.rectangle.as_ref().map(Validate::validate).transpose().map(|_| ())),
        );
        let report = report.merge(
            "tileset",
            report_of(self.tileset.as_ref().map(Validate::validate).transpose().map(|_| ())),
        );
        let report = report.merge(
            "view_from",
            report_of(self.view_from.as_ref().map(Validate::validate).transpose().map(|_| ())),
        );
        let report = report.merge("wall", report_of(self.wall.as_ref().map(Validate::validate).transpose().map(|_| ())));
        let document = self.id.as_deref() == Some("document");
        let report = match self.delete == Some(true) && self.id.is_none() {
            | true => report.merge(
                "id",
                ValidationReport::from_error("", ValidationError::new("czml_delete_id").with_message("Delete packets require an id")),
            ),
            | false => report,
        };
        let report = match !document && self.clock.is_some() {
            | true => report.merge(
                "clock",
                ValidationReport::from_error(
                    "",
                    ValidationError::new("czml_document_field").with_message("Clock belongs only on the document packet"),
                ),
            ),
            | false => report,
        };
        let report = match !document && self.version.is_some() {
            | true => report.merge(
                "version",
                ValidationReport::from_error(
                    "",
                    ValidationError::new("czml_document_field").with_message("Version belongs only on the document packet"),
                ),
            ),
            | false => report,
        };
        let report = match self.version.as_deref() {
            | Some(version) if !valid_version(version) => report.merge(
                "version",
                ValidationReport::from_error("", ValidationError::new("czml_version").with_message("CZML version must be 1.<minor>")),
            ),
            | _ => report,
        };
        report.finish()
    }
}
impl ValidateSamples for Reference {
    fn validate_samples(&self) -> Result<(), ValidationReport> {
        self.validate()
    }
}
impl<T, A: ValidateSamples> ValidateSamples for ScalarValue<T, A> {
    fn validate_samples(&self) -> Result<(), ValidationReport> {
        match self {
            | Self::Shorthand(_) => Ok(()),
            | Self::Array(array) => array.validate_samples(),
        }
    }
}
impl<T: ValidateSamples> ValidateSamples for Vec<T> {
    fn validate_samples(&self) -> Result<(), ValidationReport> {
        self.iter()
            .enumerate()
            .fold(ValidationReport::new(), |report, (index, value)| {
                report.merge(&format!("[{index}]"), report_of(value.validate_samples()))
            })
            .finish()
    }
}
impl ValidateSamples for VelocityReference {
    fn validate_samples(&self) -> Result<(), ValidationReport> {
        self.validate()
    }
}
fn increasing(left: &SampleTime, right: &SampleTime) -> bool {
    match (left, right) {
        | (SampleTime::Seconds(left), SampleTime::Seconds(right)) => left < right,
        | (SampleTime::Iso(left), SampleTime::Iso(right)) => left
            .0
            .parse::<jiff::Timestamp>()
            .ok()
            .zip(right.0.parse::<jiff::Timestamp>().ok())
            .is_some_and(|(left, right)| left < right),
        | _ => false,
    }
}
fn sample_order<T>(samples: &[(SampleTime, T)]) -> Result<(), ValidationReport> {
    let report = samples.iter().enumerate().fold(ValidationReport::new(), |report, (index, (time, _))| {
        report.merge(&format!("[{index}]"), report_of(time.validate()))
    });
    let out_of_order = samples
        .iter()
        .zip(samples.iter().skip(1))
        .any(|((left, _), (right, _))| !increasing(left, right));
    let report = if out_of_order {
        report.merge(
            "",
            ValidationReport::from_error("", ValidationError::new("czml_sample_order").with_message("Sample times must increase")),
        )
    } else {
        report
    };
    report.finish()
}
fn valid_version(value: &str) -> bool {
    value
        .split_once('.')
        .is_some_and(|(major, minor)| major == "1" && !minor.is_empty() && minor.bytes().all(|part| part.is_ascii_digit()))
}
