//! Tests against a running SurrealDB server.
//!
//! The SDK checks the server's version when it connects, so these show which server
//! versions this build of surrealdb.c accepts. Set `SURREALDB_TEST_URL` to the server's
//! HTTP endpoint (for example `http://127.0.0.1:8000`, started with `--unauthenticated`)
//! to run them; without it they are skipped.

use std::ffi::{CStr, CString};
use std::ptr;

use surrealdb_c::string::string_t;
use surrealdb_c::{Surreal, SR_ERROR};

fn server_url() -> Option<CString> {
    let url = std::env::var("SURREALDB_TEST_URL").ok()?;
    Some(CString::new(url).expect("SURREALDB_TEST_URL contains a NUL byte"))
}

fn error_message(err: &string_t) -> String {
    if err.0.is_null() {
        return String::new();
    }
    unsafe { CStr::from_ptr(err.0) }.to_string_lossy().into_owned()
}

#[test]
fn connects_to_server() {
    let Some(url) = server_url() else {
        eprintln!("SURREALDB_TEST_URL is not set; skipping");
        return;
    };
    let mut err = string_t::null();
    let mut db: *mut Surreal = ptr::null_mut();

    let res = Surreal::connect(&mut err, &mut db, url.as_ptr());
    assert_ne!(res, SR_ERROR, "connecting to {url:?} failed: {}", error_message(&err));
    assert!(res > 0, "sr_connect returned {res}: {}", error_message(&err));

    Surreal::disconnect(db);
}
