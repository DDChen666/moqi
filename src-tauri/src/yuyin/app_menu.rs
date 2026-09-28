//! The macOS menu bar menu, in the app's language. Without one, Tauri shows
//! its English default (File, Edit, View, Window, Help). The Edit menu has to
//! stay: it is what makes ⌘C / ⌘V work in the window's text fields.

use tauri::AppHandle;

#[cfg(target_os = "macos")]
pub fn apply(app: &AppHandle) {
    if let Err(e) = build(app).and_then(|menu| app.set_menu(menu).map(|_| ())) {
        log::warn!("app menu: {e}");
    }
}

#[cfg(not(target_os = "macos"))]
pub fn apply(_app: &AppHandle) {}

#[cfg(target_os = "macos")]
fn build(app: &AppHandle) -> tauri::Result<tauri::menu::Menu<tauri::Wry>> {
    use crate::tray_i18n::get_tray_translations;
    use tauri::menu::{Menu, PredefinedMenuItem as Item, Submenu};

    let locale = crate::settings::get_settings(app).app_language;
    let s = get_tray_translations(Some(locale));
    let en = get_tray_translations(Some("en".into()));
    // Locales we don't ship a menu translation for fall back to English.
    let pick = |own: &str, fallback: &str| -> String {
        if own.is_empty() { fallback } else { own }.to_string()
    };
    let t = |f: fn(&crate::tray_i18n::TrayStrings) -> &String| pick(f(&s), f(&en));

    let app_menu = Submenu::with_items(
        app,
        "Moqi",
        true,
        &[
            &Item::about(app, Some(&t(|x| &x.menu_about)), None)?,
            &Item::separator(app)?,
            &Item::hide(app, Some(&t(|x| &x.menu_hide)))?,
            &Item::hide_others(app, Some(&t(|x| &x.menu_hide_others)))?,
            &Item::show_all(app, Some(&t(|x| &x.menu_show_all)))?,
            &Item::separator(app)?,
            &Item::quit(app, Some(&t(|x| &x.menu_quit)))?,
        ],
    )?;
    let edit = Submenu::with_items(
        app,
        t(|x| &x.menu_edit),
        true,
        &[
            &Item::undo(app, Some(&t(|x| &x.menu_undo)))?,
            &Item::redo(app, Some(&t(|x| &x.menu_redo)))?,
            &Item::separator(app)?,
            &Item::cut(app, Some(&t(|x| &x.menu_cut)))?,
            &Item::copy(app, Some(&t(|x| &x.menu_copy)))?,
            &Item::paste(app, Some(&t(|x| &x.menu_paste)))?,
            &Item::select_all(app, Some(&t(|x| &x.menu_select_all)))?,
        ],
    )?;
    let window = Submenu::with_items(
        app,
        t(|x| &x.menu_window),
        true,
        &[
            &Item::minimize(app, Some(&t(|x| &x.menu_minimize)))?,
            &Item::close_window(app, Some(&t(|x| &x.menu_close_window)))?,
        ],
    )?;
    Menu::with_items(app, &[&app_menu, &edit, &window])
}
