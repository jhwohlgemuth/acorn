use acorn_macros::Validate;

#[derive(Validate)]
struct Invalid {
    #[validate(each(nested))]
    values: Vec<String>,
}

fn main() {}
