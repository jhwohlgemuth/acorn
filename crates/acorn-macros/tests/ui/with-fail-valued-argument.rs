use acorn_macros::With;

#[derive(With)]
struct Foo {
    #[with(public = true)]
    bar: String,
}

fn main() {}
