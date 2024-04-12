pub fn format_size(bytes: u64) -> String {
    match bytes {
        0..=999 => format!("{} B", bytes),
        1000..=999_999 => format!("{} KB", ((bytes as f64) / 1000.).round()),
        1_000_000..=9_999_999_999 => format!("{:.2} MB", ((bytes as f64) / 1_000_000.)),
        10_000_000_000.. => format!("{:.2} GB", ((bytes as f64) / 1_000_000_000.)),
    }
}
