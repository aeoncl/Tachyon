use std::fs;

fn scan(dir: &str, forbidden: &[&str]) {
    for entry in fs::read_dir(dir).unwrap().map(Result::unwrap) {
        let path = entry.path();
        if path.is_dir() {
            scan(path.to_str().unwrap(), forbidden);
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

#[test]
fn domain_is_pure() {
    scan(
        "src/domain",
        &[
            "tokio::",
            "async fn",
            "crate::port",
            "crate::application",
            "async_trait",
        ],
    );
}

#[test]
fn port_does_not_know_application() {
    scan("src/port", &["crate::application"]);
}
