use acorn_macros::js;

struct NotSerializable;

fn main() {
    let value = NotSerializable;
    let _ = js!("window.good({value})");
}
