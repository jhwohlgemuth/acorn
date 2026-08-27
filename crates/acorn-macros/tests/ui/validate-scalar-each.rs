use acorn_macros::Validate;

#[derive(Validate)]
struct Invalid {
    #[validate(each(email))]
    value: String,
}

fn main() {}
