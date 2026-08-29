extern crate self as acorn;

use acorn_macros::Validate;

pub mod validation {
    use core::future::{ready, Future};

    pub struct ValidationError;
    pub struct ValidationReport;

    pub trait Validate {
        fn validate(&self) -> Result<(), ValidationReport>;
        fn validate_async(&self) -> impl Future<Output = Result<(), ValidationReport>> + '_ {
            ready(self.validate())
        }
    }

    impl ValidationError {
        pub fn add_param<T: ?Sized>(&mut self, _: String, _: &T) {}
    }

    impl ValidationReport {
        pub fn new() -> Self {
            Self
        }

        pub fn add(&mut self, _: &str, _: ValidationError) {}

        pub fn finish(self) -> Result<(), Self> {
            Ok(())
        }
    }
}

async fn invalid_output(_: &String) -> bool {
    false
}

#[derive(Validate)]
struct Invalid {
    #[validate(async(function = "invalid_output"))]
    value: String,
}

fn main() {}
