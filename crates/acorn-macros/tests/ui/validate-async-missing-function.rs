use acorn_macros::Validate;

#[derive(Validate)]
struct Invalid {
    #[validate(async(code = "missing"))]
    value: String,
}

fn main() {}
