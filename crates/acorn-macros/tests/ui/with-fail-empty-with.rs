use acorn_macros::With;

#[derive(With)]
struct Foo {
    #[with()]
    bar: String,
}

fn main() {}
