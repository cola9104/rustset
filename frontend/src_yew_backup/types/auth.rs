//! Authentication state types

use shared::User;

/// Authentication state containing token and user info
#[derive(Clone, Debug, PartialEq)]
pub struct AuthState {
    pub token: Option<String>,
    pub user: Option<User>,
}
