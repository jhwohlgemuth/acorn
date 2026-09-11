use acorn_macros::With;

#[derive(With)]
#[with(unknown = "x")]
struct Foo {
    bar: String,
}

fn main() {}
