mod panel_settings;
mod titlebar_item;

use gpui::AppContext;
pub use titlebar_item::TitlebarItem;

pub fn init(cx: &mut AppContext) {
    vcs_menu::init(cx);
    titlebar_item::init(cx);
}
