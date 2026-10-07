use super::*;
#[test]
fn test_rejects_remote_plaintext_and_zero_bounds() {
    assert!(JsonRpcClient::new("http://example.com/rpc", "secret", Duration::from_secs(1), 1024).is_err());
    assert!(JsonRpcClient::new("http://127.0.0.1:8001/rpc", "secret", Duration::from_secs(1), 0).is_err());
}
#[test]
fn test_response_identifiers_must_match() {
    let response = json!({"jsonrpc":"2.0","id":2,"result":{}});
    assert!(decode_response(response, &RequestId::Number(1)).is_err());
}
