use acorn_macros::With;

#[derive(With)]
struct Foo {
    #[with(public, private)]
    bar: String,
}

fn main() {}
