pub fn remote_arg(remote: &str) -> &str {
    remote.trim_start_matches('@')
}
