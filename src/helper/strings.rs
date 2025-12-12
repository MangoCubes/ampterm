pub fn trim_long_str(s: String, to: usize) -> String {
    match s.char_indices().nth(to) {
        None => s,
        Some((idx, _)) => s[..idx].to_string(),
    }
}
