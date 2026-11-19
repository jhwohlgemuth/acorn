use super::ContentDigest;
use acorn_core::validation;

#[test]
fn digest_is_a_valid_sha256_identity() {
    let source = String::from("abc");
    let digest = source.digest();
    assert_eq!(digest, "sha256:ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
    assert_eq!(b"abc".digest(), digest);
    assert!(validation::digest(&digest).is_ok());
}
