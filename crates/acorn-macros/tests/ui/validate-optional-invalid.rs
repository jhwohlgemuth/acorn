use acorn_macros::Validate;

#[derive(Validate)]
struct Asynchronous {
    #[validate(optional(async(function = "validate")))]
    value: String,
}
#[derive(Validate)]
struct Collection {
    #[validate(optional(each(version)))]
    value: String,
}
#[derive(Validate)]
struct Empty {
    #[validate(optional())]
    value: String,
}
#[derive(Validate)]
struct Multiple {
    #[validate(optional(version, email))]
    value: String,
}
#[derive(Validate)]
struct Nested {
    #[validate(optional(nested))]
    value: String,
}
#[derive(Validate)]
struct Recursive {
    #[validate(optional(optional(version)))]
    value: String,
}
#[derive(Validate)]
struct Required {
    #[validate(optional(required))]
    value: Option<String>,
}
#[derive(Validate)]
struct Unsupported {
    #[validate(optional(range(min = 1)))]
    value: u8,
}
fn main() {}
