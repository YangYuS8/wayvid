#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CoverArtSource {
    Bundled(String),
    Placeholder,
}

pub fn cover_art_source(bundled_cover_path: Option<String>) -> CoverArtSource {
    match bundled_cover_path {
        Some(path) if !path.trim().is_empty() => CoverArtSource::Bundled(path),
        _ => CoverArtSource::Placeholder,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shared_policy_prefers_bundled_cover_before_placeholder() {
        assert_eq!(
            cover_art_source(Some("covers/forest.png".to_string())),
            CoverArtSource::Bundled("covers/forest.png".to_string())
        );
        assert_eq!(cover_art_source(None), CoverArtSource::Placeholder);
    }
}
