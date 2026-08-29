use acorn_macros::Validate;

#[derive(Validate)]
struct Invalid {
    #[validate(not(required))]
    value: Option<String>,
}

fn main() {}
