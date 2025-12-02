use base64::{Engine as _, engine::general_purpose};
use insta::assert_snapshot;

/// Helper function to snapshot image bytes with insta
/// Converts bytes to base64 for snapshotting
pub fn snapshot_image_bytes(image_bytes: &[u8], snapshot_name: &str) {
    let base64_str = general_purpose::STANDARD.encode(image_bytes);
    assert_snapshot!(snapshot_name, base64_str);
}
