pub fn trim_long_str(mut s: String, to: usize) -> String {
    let len = s.len();
    if len > to {
        s.truncate(to - 3);
        s.push_str("...");
    }
    s
}
