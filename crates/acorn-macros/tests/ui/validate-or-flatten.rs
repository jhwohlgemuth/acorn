use acorn_macros::Validate;

#[derive(Validate)]
struct Invalid {
    #[validate(or(email, or(url, nonempty, code = "nested")))]
    value: String,
}

fn main() {}
