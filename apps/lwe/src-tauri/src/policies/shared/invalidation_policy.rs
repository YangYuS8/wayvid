use crate::action_outcome::InvalidatedPage;

pub fn pages_after_workshop_refresh() -> Vec<InvalidatedPage> {
    vec![InvalidatedPage::Library]
}
