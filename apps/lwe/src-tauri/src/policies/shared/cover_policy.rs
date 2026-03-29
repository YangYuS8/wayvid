pub fn bundled_cover_or_none(cover_path: Option<String>) -> Option<String> {
    cover_path.filter(|value| !value.trim().is_empty())
}
