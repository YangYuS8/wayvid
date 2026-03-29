use crate::results::desktop::DesktopPageResult;

pub struct DesktopService;

impl DesktopService {
    pub fn load_page() -> Result<DesktopPageResult, String> {
        Ok(DesktopPageResult { stale: true })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn desktop_service_returns_stale_placeholder_result() {
        let result = DesktopService::load_page().unwrap();
        let DesktopPageResult { stale } = result;

        assert!(stale);
    }
}
