use acorn_macros::Validate;

#[derive(Validate)]
struct Invalid {
    #[validate(or(email, required))]
    value: String,
}

fn main() {}
