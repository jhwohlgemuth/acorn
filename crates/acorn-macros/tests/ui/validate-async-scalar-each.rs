use acorn_macros::Validate;

#[derive(Validate)]
struct Invalid {
    #[validate(each(async(function = "validate_value")))]
    value: String,
}

fn main() {}
