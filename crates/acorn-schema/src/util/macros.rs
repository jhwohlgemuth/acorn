#[macro_export]
macro_rules! impl_to_markdown {
    ($($type:ty),+ $(,)?) => {
        $(impl acorn_core::util::MarkdownSupport for $type {
            fn to_markdown(&self) -> String {
                self.to_string()
            }
        })+
    };
}
#[macro_export]
macro_rules! regex_validator {
    ($(#[$meta:meta])* $fn_name:ident, $regex:expr, $code:literal, $message:literal) => {
        #[doc = concat!("Check if value is a valid ", $code)]
        $(#[$meta])*
        pub fn $fn_name(value: &str) -> Result<(), acorn_core::validation::ValidationError> {
            match $regex.is_match(value) {
                | Ok(true) => Ok(()),
                | _ => Err(acorn_core::validation::ValidationError::new($code).with_message($message)),
            }
        }
    };
    ($(#[$meta:meta])* $fn_name:ident, $regex:expr, $code:literal) => {
        #[doc = concat!("Check if value is a valid ", $code)]
        $(#[$meta])*
        pub fn $fn_name(value: &str) -> Result<(), acorn_core::validation::ValidationError> {
            match $regex.is_match(value) {
                | Ok(true) => Ok(()),
                | _ => Err(acorn_core::validation::ValidationError::new($code)
                    .with_message(concat!("Provide valid ", $code))),
            }
        }
    };
}
