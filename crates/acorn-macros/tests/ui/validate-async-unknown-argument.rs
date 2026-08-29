use acorn_macros::Validate;

#[derive(Validate)]
struct Invalid {
    #[validate(async(function = "validate_value", unknown = true))]
    value: String,
}

fn main() {}
