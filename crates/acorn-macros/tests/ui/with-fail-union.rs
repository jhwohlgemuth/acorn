use acorn_macros::With;

#[derive(With)]
union Foo {
    integer: u32,
    float: f32,
}

fn main() {}
