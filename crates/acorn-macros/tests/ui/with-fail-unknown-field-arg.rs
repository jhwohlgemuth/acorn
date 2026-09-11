use acorn_macros::With;

#[derive(With)]
struct Foo {
    #[with(unknown)]
    bar: String,
}

fn main() {}
