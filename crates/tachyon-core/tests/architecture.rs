use std::fs;
use std::path::Path;

fn scan(dir: &str, forbidden: &[&str]) {
    let path = Path::new(dir);
    assert!(path.is_dir(), "{} must exist for this test to mean anything", dir);
    scan_dir(path, forbidden);
}

fn scan_dir(dir: &Path, forbidden: &[&str]) {
    for entry in fs::read_dir(dir).unwrap().map(Result::unwrap) {
        let path = entry.path();
        if path.is_dir() {
            scan_dir(&path, forbidden);
            continue;
        }
        let src = fs::read_to_string(&path).unwrap();
        for needle in forbidden {
            assert!(
                !src.contains(needle),
                "{} must not contain `{}`",
                path.display(),
                needle
            );
        }
    }
}

/// The domain is plain data and rules: no runtime, no async, no knowledge of the layers
/// above it.
#[test]
fn domain_is_pure() {
    scan(
        "src/domain",
        &[
            "tokio::",
            "async fn",
            "crate::application",
            "crate::infrastructure",
            "async_trait",
        ],
    );
}

/// Ports and use cases are defined without reference to the wiring that instantiates them.
#[test]
fn application_does_not_know_infrastructure() {
    scan("src/application", &["crate::infrastructure"]);
}
