mod collab_titlebar_item;
mod panel_settings;

pub use collab_titlebar_item::CollabTitlebarItem;
use gpui::AppContext;
pub use panel_settings::NotificationPanelSettings;

pub fn init(cx: &mut AppContext) {
    vcs_menu::init(cx);
    collab_titlebar_item::init(cx);
}
