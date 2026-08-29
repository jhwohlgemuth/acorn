use acorn_macros::Validate;

#[derive(Validate)]
#[validate(schema(function = "check", typo = "x"))]
struct Invalid {
    value: String,
}

fn main() {}
