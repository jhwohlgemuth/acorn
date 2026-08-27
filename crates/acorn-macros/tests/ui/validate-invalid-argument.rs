use acorn_macros::Validate;

#[derive(Validate)]
struct Invalid {
    #[validate(length(minimum = 1))]
    values: Vec<String>,
}

fn main() {}
