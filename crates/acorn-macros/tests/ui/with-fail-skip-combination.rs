use acorn_macros::With;

#[derive(With)]
struct Foo {
    #[with(skip, some)]
    bar: Option<String>,
}

fn main() {}
