use acorn_macros::Validate;

#[derive(Validate)]
struct Invalid {
    #[validate(not(email, url))]
    value: String,
}

fn main() {}
