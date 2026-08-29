use acorn_macros::Validate;

#[derive(Validate)]
#[validate(schema(function = "check", if = "self.enabled &&"))]
struct Invalid {
    enabled: bool,
}

fn main() {}
