pub(crate) mod api;

use crate::core::api::MOCK_USER_ID;

pub(crate) struct AppleSession {
    pub(crate) token: &'static str,
    pub(crate) user_id: &'static str,
}

pub(crate) fn apple_login() -> AppleSession {
    AppleSession {
        token: "mock",
        user_id: MOCK_USER_ID,
    }
}
