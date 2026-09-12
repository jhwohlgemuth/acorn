mod fixture {
    use acorn_macros::With;

    #[derive(With)]
    pub struct Foo {
        #[with(private)]
        pub bar: String,
    }
}

fn main() {
    let _ = fixture::Foo { bar: String::new() }.with_bar("private");
}
