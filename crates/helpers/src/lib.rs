pub fn parse_pid_from_bytes(bytes: &[u8]) -> Option<i32> {
    if bytes.is_empty() || bytes.len() > 10 {
        return None;
    }

    let mut result: i32 = 0;
    for &b in bytes {
        if !b.is_ascii_digit() {
            return None;
        }
        result = result.checked_mul(10)?.checked_add((b - b'0').into())?;
    }
    if result == 0 { None } else { Some(result) }
}
