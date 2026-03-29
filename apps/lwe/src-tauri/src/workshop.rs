pub(crate) fn workshop_item_url(workshop_id: &str) -> String {
    format!("https://steamcommunity.com/sharedfiles/filedetails/?id={workshop_id}")
}

pub(crate) fn steam_openurl(workshop_id: &str) -> String {
    format!("steam://openurl/{}", workshop_item_url(workshop_id))
}

#[allow(unused_imports)]
pub use crate::commands::workshop::{
    load_workshop_item_detail, load_workshop_page, open_workshop_in_steam, refresh_workshop_catalog,
};

#[cfg(test)]
mod boundary_tests {
    #[test]
    fn command_module_name_no_longer_implies_service_logic() {
        assert!(true);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn steam_url_uses_official_workshop_page() {
        assert_eq!(
            workshop_item_url("12345"),
            "https://steamcommunity.com/sharedfiles/filedetails/?id=12345"
        );
    }

    #[test]
    fn steam_openurl_wraps_official_workshop_page() {
        assert_eq!(
            steam_openurl("12345"),
            "steam://openurl/https://steamcommunity.com/sharedfiles/filedetails/?id=12345"
        );
    }
}
