pub fn is_builtin_system_account(username: &str) -> bool {
    matches!(username, "admin" | "sec" | "audit")
}
