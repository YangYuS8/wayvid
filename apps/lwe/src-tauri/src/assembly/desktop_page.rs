use crate::models::DesktopPageSnapshot;
use crate::results::desktop::DesktopPageResult;

pub fn assemble_desktop_page(result: DesktopPageResult) -> DesktopPageSnapshot {
    DesktopPageSnapshot {
        monitors: Vec::new(),
        stale: result.stale,
    }
}
