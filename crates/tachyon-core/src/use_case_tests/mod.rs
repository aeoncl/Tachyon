//! Use-case tests, wired over the real in-memory repositories with the
//! `application::test_support` fakes standing in for backends and stores.
//!
//! They live outside `src/application` because the `application_does_not_know_infrastructure`
//! architecture test forbids that module from naming `crate::infrastructure`, and testing a
//! use case against a second, test-only `SessionRepository` would leave the real one, and the
//! readiness rule it enforces, untested.

mod auth;
mod device_verification;
