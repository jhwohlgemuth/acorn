use acorn_macros::Validate;

#[derive(Validate)]
struct Invalid {
    #[validate(unknown)]
    value: String,
}

fn main() {}
