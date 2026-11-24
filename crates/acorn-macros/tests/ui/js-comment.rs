use acorn_macros::js;

fn main() {
    let value = "x";
    let _ = js!("// {value}\nwindow.good()");
}
