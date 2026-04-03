use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub enum DesktopPersistenceLoad {
    Loaded(BTreeMap<String, String>),
    Unavailable { reason: String },
}

#[derive(Debug, Clone)]
pub enum DesktopPersistenceWrite {
    Saved,
    Cleared,
    Unavailable { reason: String },
}
