use acorn_macros::Validate;

#[derive(Validate)]
struct Invalid {
    #[validate(each(each(email)))]
    values: Vec<String>,
}

fn main() {}
