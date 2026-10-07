use super::*;

#[test]
fn test_non_loopback_listeners_require_tls() {
    let remote = Some(SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 8001));
    assert!(validate_listener_security(remote, None, false).is_err());
    assert!(validate_listener_security(remote, None, true).is_ok());
    assert!(validate_listener_security(Some(DEFAULT_JSON_RPC_BIND), Some(DEFAULT_GRPC_BIND), false).is_ok());
}
#[test]
fn test_numeric_limits_must_be_positive() {
    assert!(positive_u64(Some(0), "TEST_SECONDS", 30).is_err());
    assert!(positive_usize(Some(0), "TEST_BYTES", 1024).is_err());
}
