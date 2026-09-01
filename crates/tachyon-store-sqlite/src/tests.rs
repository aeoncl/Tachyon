use crate::SqliteStore;
use tachyon_core::application::ports::{AccountRepository, CredentialRepository};
use tachyon_core::domain::auth::{CredentialBlob, TachyonToken};
use tachyon_core::domain::ids::LoginId;

fn token(s: &str) -> TachyonToken {
    TachyonToken::new(s)
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
async fn relinking_a_token_drops_the_superseded_login() {
    let store = SqliteStore::open_in_memory().unwrap();
    store.store(&login("old"), blob(b"old-creds")).await.unwrap();
    store.save_login_for_token(token("t1"), login("old")).await.unwrap();

    store.store(&login("new"), blob(b"new-creds")).await.unwrap();
    store.save_login_for_token(token("t1"), login("new")).await.unwrap();

    assert_eq!(
        store.login_id_by_token(&token("t1")).await.unwrap(),
        Some(login("new"))
    );
    assert_eq!(store.credentials(&login("old")).await.unwrap(), None);
    assert_eq!(
        store.credentials(&login("new")).await.unwrap(),
        Some(blob(b"new-creds"))
    );
}

#[tokio::test]
async fn superseded_login_survives_while_another_token_references_it() {
    let store = SqliteStore::open_in_memory().unwrap();
    store.store(&login("shared"), blob(b"creds")).await.unwrap();
    store.save_login_for_token(token("t1"), login("shared")).await.unwrap();
    store.save_login_for_token(token("t2"), login("shared")).await.unwrap();

    store.save_login_for_token(token("t1"), login("other")).await.unwrap();

    assert_eq!(
        store.credentials(&login("shared")).await.unwrap(),
        Some(blob(b"creds"))
    );
    assert_eq!(
        store.login_id_by_token(&token("t2")).await.unwrap(),
        Some(login("shared"))
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
