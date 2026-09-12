use acorn_macros::With;

#[derive(With)]
struct Foo {
    #[with(some, some)]
    bar: Option<String>,
}

fn main() {}
