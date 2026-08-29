use acorn_macros::Validate;

#[derive(Validate)]
struct Invalid {
    #[validate(not(not(email)))]
    value: String,
}

fn main() {}
