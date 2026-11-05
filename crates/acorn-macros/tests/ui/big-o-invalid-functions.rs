use acorn_macros::bigo;

fn main() {
    let _ = bigo!("O(log_1(n))");
    let _ = bigo!("O(polylog(n-m))");
    let _ = bigo!("O(W_out(n");
}
