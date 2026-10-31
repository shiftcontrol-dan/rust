//! Points the PropelAuth backend API base URL at a local server in tests.
//!
//! This module exists only with the `test-support` feature, which belongs in a
//! consumer's `[dev-dependencies]`. With the feature on, `PropelAuth::init` and
//! `PropelAuth::fetch_and_init` read the URL set here instead of
//! `https://propelauth-api.com`, and panic when none is set. A test that forgets
//! to set a URL therefore fails instead of calling PropelAuth.
//!
//! The URL is read when `init` or `fetch_and_init` runs. A `PropelAuth` value keeps
//! that URL afterwards, even if the override changes.
//!
//! The override is process-wide. Tests that set different URLs must not run
//! at the same time: serialize them with a lock, or run each in its own process.

use std::sync::{PoisonError, RwLock};

static BASE_URL: RwLock<Option<String>> = RwLock::new(None);

/// Sends the backend API base URL to `url`, for example a mock server's URI.
pub fn set_base_url(url: impl Into<String>) {
    *BASE_URL.write().unwrap_or_else(PoisonError::into_inner) = Some(url.into());
}

/// Removes the URL that [`set_base_url`] set. Initialization then panics.
pub fn clear_base_url() {
    *BASE_URL.write().unwrap_or_else(PoisonError::into_inner) = None;
}

pub(crate) fn base_url_or_panic() -> String {
    let url = BASE_URL
        .read()
        .unwrap_or_else(PoisonError::into_inner)
        .clone();
    url.expect(
        "propelauth test-support: a test must call \
         propelauth::test_support::set_base_url before it initializes PropelAuth",
    )
}

#[cfg(test)]
pub(crate) fn test_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    LOCK.lock().unwrap_or_else(PoisonError::into_inner)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base_url_reads_the_override() {
        let _lock = test_lock();
        set_base_url("http://127.0.0.1:9");
        assert_eq!(base_url_or_panic(), "http://127.0.0.1:9");
        clear_base_url();
    }

    #[test]
    #[should_panic(expected = "must call propelauth::test_support::set_base_url")]
    fn base_url_panics_when_no_override_is_set() {
        let _lock = test_lock();
        clear_base_url();
        base_url_or_panic();
    }

    #[test]
    fn clear_base_url_removes_the_override() {
        let _lock = test_lock();
        set_base_url("http://127.0.0.1:9");
        clear_base_url();
        assert!(std::panic::catch_unwind(base_url_or_panic).is_err());
    }
}
