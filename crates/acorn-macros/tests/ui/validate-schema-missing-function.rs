use acorn_macros::Validate;

#[derive(Validate)]
#[validate(schema(skip_on_field_errors = false))]
struct Invalid {
    value: String,
}

fn main() {}
