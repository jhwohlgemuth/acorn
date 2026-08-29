use acorn_macros::Validate;

#[derive(Validate)]
struct Invalid {
    #[validate(or(email))]
    value: String,
}

fn main() {}
