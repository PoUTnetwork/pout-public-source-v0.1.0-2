//! Public extraction regressions. A test definition is not an executed result.
#[allow(dead_code)]
#[path = "../examples/support/retrieval_observation.rs"]
mod retrieval;
#[allow(dead_code)]
#[path = "../examples/support/observation_fixture.rs"]
mod fixture;
use retrieval::{CheckedResponse, Error, Manifest, PolicyError, CHUNK_BYTES, HEADER_BYTES};
use std::{
    fs,
    sync::{atomic::AtomicBool, OnceLock},
};

fn manifest() -> &'static Manifest {
    static VALUE: OnceLock<Manifest> = OnceLock::new();
    VALUE.get_or_init(fixture::manifest)
}
fn no() -> AtomicBool {
    AtomicBool::new(false)
}
fn request() -> retrieval::Request {
    fixture::request(manifest(), 0)
}
fn frame(request: &retrieval::Request) -> Vec<u8> {
    let mut result = Vec::with_capacity(HEADER_BYTES + CHUNK_BYTES);
    result.extend_from_slice(b"POUTRET1");
    result.push(0);
    result.extend_from_slice(&request.digest().unwrap());
    result.extend_from_slice(&(CHUNK_BYTES as u32).to_be_bytes());
    result.extend_from_slice(&fixture::chunk(request.chunk));
    result
}
fn verified(request: &retrieval::Request) -> retrieval::VerifiedResponse {
    match retrieval::verify_one(manifest(), request, &mut frame(request).as_slice(), &no()).unwrap() {
        CheckedResponse::Verified(value) => value,
        CheckedResponse::Unavailable => panic!("fixture must contain data"),
    }
}

#[test]
fn public_fixture_matches_independent_python_known_answers() {
    assert_eq!(fixture::hex(&manifest().object_id), "3243d65f4db13b71fb7f331d0bffb20207274ae6e88dc569c8b7b47a67edbedd");
    assert_eq!(fixture::hex(&request().digest().unwrap()), "73337bdcd65bc7297edb81d28f88fb51d9d5563223632ac4d41f44e66fefa8dc");
    assert_eq!(fixture::hex(&manifest().chunk_hashes[0]), "4eebf626d70aa281a3474fe86e9fe9f302c964e43982abccb1e791b751d67c5a");
}
#[test]
fn exact_frame_verifies_and_payload_is_not_header_bytes() {
    let request = request();
    let result = retrieval::verify_stream(manifest(), &[request], &mut frame(&request).as_slice(), &no()).unwrap();
    assert_eq!((result.verified, result.unavailable), (1, 0));
    assert_eq!(result.transferred_bytes, CHUNK_BYTES as u64);
}
#[test]
fn corrupted_payload_is_rejected() {
    let request = request();
    let mut bytes = frame(&request);
    bytes[HEADER_BYTES + 101] ^= 1;
    assert_eq!(retrieval::verify_one(manifest(), &request, &mut bytes.as_slice(), &no()), Err(Error::Integrity));
}
#[test]
fn truncation_and_trailing_bytes_are_framing_errors() {
    let request = request();
    let mut bytes = frame(&request);
    bytes.pop();
    assert_eq!(retrieval::verify_stream(manifest(), &[request], &mut bytes.as_slice(), &no()), Err(Error::Framing));
    bytes = frame(&request);
    bytes.push(0);
    assert_eq!(retrieval::verify_stream(manifest(), &[request], &mut bytes.as_slice(), &no()), Err(Error::Framing));
}
#[test]
fn original_response_does_not_match_a_fresh_nonce() {
    let request = request();
    let bytes = frame(&request);
    let mut next = request;
    next.nonce[0] ^= 1;
    assert_eq!(retrieval::verify_one(manifest(), &next, &mut bytes.as_slice(), &no()), Err(Error::RequestContext));
}
#[test]
fn correctly_retagged_cached_payload_remains_valid() {
    let mut request = request();
    request.nonce[0] ^= 1;
    assert_eq!(verified(&request).returned_bytes(), CHUNK_BYTES as u64);
}
#[test]
fn expected_object_must_match_independent_manifest() {
    let mut request = request();
    request.object_id[0] ^= 1;
    assert_eq!(retrieval::verify_one(manifest(), &request, &mut frame(&request).as_slice(), &no()), Err(Error::ObjectPolicy));
}
#[test]
fn every_request_context_field_changes_the_request_digest() {
    let base = request();
    let original = base.digest().unwrap();
    for field in 0..6 {
        let mut changed = base;
        match field {
            0 => changed.object_id[0] ^= 1,
            1 => changed.label = 1,
            2 => changed.interval = 1,
            3 => changed.sample = 1,
            4 => changed.chunk = 1,
            _ => changed.nonce[0] ^= 1,
        }
        assert_ne!(changed.digest().unwrap(), original);
    }
}
#[test]
fn restricted_state_only_response_is_unavailable_not_invalid_data() {
    let request = request();
    let mut bytes = Vec::new();
    retrieval::respond_state_only(&[request], &mut bytes, &no()).unwrap();
    assert_eq!(bytes.len(), HEADER_BYTES);
    let result = retrieval::verify_stream(manifest(), &[request], &mut bytes.as_slice(), &no()).unwrap();
    assert_eq!((result.verified, result.unavailable, result.transferred_bytes), (0, 1, 0));
}
#[test]
fn invalid_frame_status_length_and_magic_reject() {
    let request = request();
    for offset in [0, 8, 44] {
        let mut bytes = frame(&request);
        bytes[offset] = 255;
        assert_eq!(retrieval::verify_one(manifest(), &request, &mut bytes.as_slice(), &no()), Err(Error::Framing));
    }
}
#[test]
fn local_consumption_is_not_demand_or_global_history() {
    let request = request();
    let value = verified(&request);
    let mut retained = retrieval::Consumption::new().unwrap();
    assert!(retained.is_empty());
    assert_eq!(retained.admit(&value, 7, 8), Ok(()));
    assert_eq!(retained.admit(&value, 7, 8), Err(PolicyError::Duplicate));
    assert_eq!(retained.admit(&value, 8, 8), Err(PolicyError::Expired));
    let mut copied_empty_view = retrieval::Consumption::new().unwrap();
    assert_eq!(copied_empty_view.admit(&value, 7, 8), Ok(()));
    let mut next = request;
    next.nonce[0] ^= 1;
    assert_eq!(retained.admit(&verified(&next), 7, 8), Ok(()));
    assert_eq!(retained.len(), 2); // Fresh nonces are NOT distinct-demand certificates.
}
#[test]
fn bounds_cancellation_and_manifest_mismatch_fail_explicitly() {
    assert_eq!(retrieval::validate_requests(&[]), Err(Error::Bounds));
    assert_eq!(retrieval::validate_requests(&[request(); 9]), Err(Error::Bounds));
    let mut bad = request();
    bad.chunk = 32;
    assert_eq!(bad.validate(), Err(Error::Bounds));
    let mut altered = manifest().clone();
    altered.chunk_hashes[0][0] ^= 1;
    assert_eq!(altered.validate(), Err(Error::ObjectPolicy));
    assert_eq!(retrieval::verify_one(manifest(), &request(), &mut &[][..], &AtomicBool::new(true)), Err(Error::Cancelled));
}
#[test]
fn actual_file_read_matches_public_bytes_and_never_overwrites() {
    let root = std::env::temp_dir().join(format!("pout-public-observation-{}", std::process::id()));
    fs::create_dir(&root).expect("fresh exclusively owned test directory");
    struct Cleanup(std::path::PathBuf);
    impl Drop for Cleanup {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    let _cleanup = Cleanup(root.clone());
    let source = root.join("synthetic.bin");
    fixture::write_source(&source).unwrap();
    assert!(fixture::write_source(&source).is_err());
    let request = request();
    let mut bytes = Vec::new();
    retrieval::respond_file(&[request], &source, &mut bytes, &no()).unwrap();
    assert_eq!(bytes, frame(&request));
    #[cfg(unix)]
    {
        let alias = root.join("symlink.bin");
        std::os::unix::fs::symlink(&source, &alias).unwrap();
        assert!(matches!(retrieval::open_regular(&alias), Err(Error::UnsafePath)));
    }
}
