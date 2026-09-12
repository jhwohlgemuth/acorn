use acorn_macros::With;

#[derive(With)]
struct Foo {
    #[with(some)]
    bar: String,
}

fn main() {}
