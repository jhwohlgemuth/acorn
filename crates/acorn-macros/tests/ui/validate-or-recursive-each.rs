use acorn_macros::Validate;

#[derive(Validate)]
struct Invalid {
    #[validate(or(email, each(url)))]
    value: String,
}

fn main() {}
