use crate::SqliteStore;
use tachyon_core::application::ports::{AccountRepository, CredentialRepository, StoredLogin};
use tachyon_core::domain::auth::{CredentialBlob, BridgeLinkToken};
use tachyon_core::domain::ids::LoginId;

fn token(s: &str) -> BridgeLinkToken {
    BridgeLinkToken::new(s)
}

fn login(s: &str) -> LoginId {
    LoginId::new(s)
}

fn blob(bytes: &[u8]) -> CredentialBlob {
    CredentialBlob::new(bytes.to_vec())
}

#[tokio::test]
async fn linked_token_resolves_to_its_login() {
    let store = SqliteStore::open_in_memory().unwrap();

    store.save_login_for_token(token("t1"), login("l1")).await.unwrap();

    assert_eq!(
        store.login_id_by_token(&token("t1")).await.unwrap(),
        Some(login("l1"))
    );
    assert_eq!(store.login_id_by_token(&token("unknown")).await.unwrap(), None);
}

#[tokio::test]
async fn stored_credentials_round_trip_and_latest_wins() {
    let store = SqliteStore::open_in_memory().unwrap();

    assert_eq!(store.credentials(&login("l1")).await.unwrap(), None);

    store.store(&login("l1"), blob(b"first")).await.unwrap();
    store.store(&login("l1"), blob(b"second")).await.unwrap();

    assert_eq!(
        store.credentials(&login("l1")).await.unwrap(),
        Some(blob(b"second"))
    );
}

#[tokio::test]
async fn deleting_a_login_takes_its_tokens_with_it() {
    let store = SqliteStore::open_in_memory().unwrap();
    store.store(&login("l1"), blob(b"creds")).await.unwrap();
    store.save_login_for_token(token("t1"), login("l1")).await.unwrap();
    store.save_login_for_token(token("t2"), login("l1")).await.unwrap();

    store.delete_login(&login("l1")).await.unwrap();

    assert_eq!(store.credentials(&login("l1")).await.unwrap(), None);
    assert_eq!(store.login_id_by_token(&token("t1")).await.unwrap(), None);
    assert_eq!(store.login_id_by_token(&token("t2")).await.unwrap(), None);
    assert!(store.logins().await.unwrap().is_empty());
}

#[tokio::test]
async fn deleting_a_login_that_is_not_there_is_fine() {
    let store = SqliteStore::open_in_memory().unwrap();

    store.delete_login(&login("nope")).await.unwrap();
}

#[tokio::test]
async fn logins_report_whether_a_token_still_points_at_them() {
    let store = SqliteStore::open_in_memory().unwrap();
    store.store(&login("bound"), blob(b"creds")).await.unwrap();
    store.save_login_for_token(token("t1"), login("bound")).await.unwrap();
    store.store(&login("leftover"), blob(b"creds")).await.unwrap();

    let mut logins = store.logins().await.unwrap();
    logins.sort_by(|a, b| a.login_id.to_string().cmp(&b.login_id.to_string()));

    assert_eq!(
        logins,
        vec![
            StoredLogin { login_id: login("bound"), linked: true },
            StoredLogin { login_id: login("leftover"), linked: false },
        ]
    );
}

#[tokio::test]
async fn relinking_to_the_same_login_is_idempotent() {
    let store = SqliteStore::open_in_memory().unwrap();
    store.store(&login("l1"), blob(b"creds")).await.unwrap();

    store.save_login_for_token(token("t1"), login("l1")).await.unwrap();
    store.save_login_for_token(token("t1"), login("l1")).await.unwrap();

    assert_eq!(
        store.credentials(&login("l1")).await.unwrap(),
        Some(blob(b"creds"))
    );
}

#[tokio::test]
async fn a_reopened_store_still_holds_everything() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("tachyon.db");

    {
        let store = SqliteStore::open(&path).unwrap();
        store.store(&login("l1"), blob(b"creds")).await.unwrap();
        store.save_login_for_token(token("t1"), login("l1")).await.unwrap();
        assert_eq!(store.user_version(), crate::schema::MIGRATIONS.len() as i64);
    } // dropped: on Windows the open connection would block the reopen and the cleanup

    let store = SqliteStore::open(&path).unwrap();
    assert_eq!(store.user_version(), crate::schema::MIGRATIONS.len() as i64);
    assert_eq!(
        store.login_id_by_token(&token("t1")).await.unwrap(),
        Some(login("l1"))
    );
    assert_eq!(
        store.credentials(&login("l1")).await.unwrap(),
        Some(blob(b"creds"))
    );
}
