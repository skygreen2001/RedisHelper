#![cfg_attr(all(not(debug_assertions), target_os = "windows"), windows_subsystem = "windows")]

mod commands;
mod redis;
mod storage;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri::{Builder, Emitter, Manager, menu::{MenuBuilder, MenuItem, MenuItemBuilder, PredefinedMenuItem, SubmenuBuilder, AboutMetadata}, Wry};

/// 菜单状态管理结构
struct MenuState {
    toggle_tab_bar_items: Arc<Mutex<HashMap<String, MenuItem<Wry>>>>,
}

impl MenuState {
    fn new() -> Self {
        Self {
            toggle_tab_bar_items: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

/// 为 macOS 菜单项设置 SF Symbols 图标
// SF Symbols 图标设置函数已移除，等待 Tauri 2.x API 支持

#[tauri::command]
fn create_new_window(app: tauri::AppHandle) -> Result<(), String> {
    use tauri::WebviewWindowBuilder;
    let window_id = format!("window_{}", chrono::Utc::now().timestamp_millis());
    WebviewWindowBuilder::new(
        &app,
        &window_id,
        tauri::WebviewUrl::App("index.html?window=true".into()),
    )
    .title("Redis小助手")
    .inner_size(1000.0, 800.0)
    .resizable(true)
    .build()
    .map_err(|e| e.to_string())?;
    Ok(())
}

/// 更新标签栏菜单的文本
#[tauri::command]
fn update_toggle_tab_bar_menu(
    window: tauri::Window,
    show_tab_bar: bool,
    state: tauri::State<MenuState>,
) -> Result<(), String> {
    let items = state.toggle_tab_bar_items.lock().unwrap();
    if let Some(menu_item) = items.get(window.label()) {
        let text = if show_tab_bar { "隐藏标签栏" } else { "显示标签栏" };
        menu_item.set_text(text).map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn main() {
    Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .manage(MenuState::new())
        .setup(|app| {
            // 创建中文菜单
            let toggle_tab_bar_item = MenuItemBuilder::with_id("toggle_tab_bar", "隐藏标签栏")
                .accelerator("CmdOrCtrl+Shift+T")
                .build(app)?;

            let new_window_item = MenuItemBuilder::with_id("new_window", "新建窗口")
                .accelerator("CmdOrCtrl+N")
                .build(app)?;

            // 为菜单项设置 SF Symbols 图标（暂时注释，等待 Tauri 2.x API 支持）
            // set_menu_item_icon(&new_window_item, "doc.on.doc");
            // set_menu_item_icon(&toggle_tab_bar_item, "rectangle.bottomthird.inset.filled");

            // 保存菜单项到状态
            {
                let state = app.state::<MenuState>();
                let mut items = state.toggle_tab_bar_items.lock().unwrap();
                // 初始窗口标签固定为 "main"
                items.insert("main".to_string(), toggle_tab_bar_item.clone());
            }

            let menu = MenuBuilder::new(app)
                .item(
                    &SubmenuBuilder::new(app, "Redis小助手")
                        .item(&PredefinedMenuItem::about(app, Some("关于"), Some(AboutMetadata {
                            name: Some("Redis小助手".to_string()),
                            version: Some(env!("CARGO_PKG_VERSION").to_string()),
                            short_version: None,
                            authors: None,
                            comments: None,
                            copyright: None,
                            license: None,
                            website: None,
                            website_label: None,
                            credits: None,
                            icon: None,
                        }))?)
                        .separator()
                        .item(&PredefinedMenuItem::hide(app, Some("隐藏"))?)
                        .item(&PredefinedMenuItem::hide_others(app, Some("隐藏其他"))?)
                        .item(&PredefinedMenuItem::show_all(app, Some("显示全部"))?)
                        .separator()
                        .item(&PredefinedMenuItem::quit(app, Some("退出"))?)
                        .build()?
                )
                .item(
                    &SubmenuBuilder::new(app, "窗口")
                        .item(&new_window_item)
                        .separator()
                        .item(&PredefinedMenuItem::close_window(app, Some("关闭窗口"))?)
                        .separator()
                        .item(&toggle_tab_bar_item)
                        .item(&PredefinedMenuItem::minimize(app, Some("最小化"))?)
                        .item(&PredefinedMenuItem::fullscreen(app, Some("全屏"))?)
                        .separator()
                        .build()?
                )
                .item(
                    &SubmenuBuilder::new(app, "帮助")
                        .item(&MenuItemBuilder::with_id("help_doc", "帮助文档").build(app)?)
                        .item(&MenuItemBuilder::with_id("about", "关于").build(app)?)
                        .build()?
                )
                .build()?;

            app.set_menu(menu)?;

            // 监听菜单点击事件，向当前活动窗口发送
            app.on_menu_event(move |app, event| {
                let id = event.id().0.as_str();
                for (_label, window) in app.webview_windows() {
                    if window.is_visible().unwrap_or(false) && window.is_focused().unwrap_or(false) {
                        let _ = window.emit("menu-event", id);
                        break;
                    }
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::server::add_server,
            commands::server::edit_server,
            commands::server::delete_server,
            commands::server::get_servers,
            commands::server::test_connection,
            commands::redis::connect,
            commands::redis::get_databases,
            commands::redis::get_keys,
            commands::redis::get_key_value,
            commands::redis::set_key_value,
            commands::redis::delete_key,
            commands::redis::search_keys,
            commands::redis::create_database,
            commands::redis::delete_database,
            commands::redis::flush_database,
            commands::redis::generate_test_data,
            commands::export::export_data,
            commands::export::import_data,
            commands::trash::move_to_trash,
            commands::trash::batch_move_to_trash,
            commands::trash::get_trash_items,
            commands::trash::restore_from_trash,
            commands::trash::batch_restore_from_trash,
            commands::trash::permanent_delete_trash,
            commands::trash::clear_expired_trash,
            create_new_window,
            update_toggle_tab_bar_menu
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
