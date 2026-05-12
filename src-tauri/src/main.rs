#![cfg_attr(all(not(debug_assertions), target_os = "windows"), windows_subsystem = "windows")]

mod commands;
mod redis;
mod storage;

use tauri::{Builder, menu::{MenuBuilder, MenuItemBuilder, PredefinedMenuItem, SubmenuBuilder, AboutMetadata}};

fn main() {
    Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            // 创建中文菜单
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
                    &SubmenuBuilder::new(app, "编辑")
                        .item(&PredefinedMenuItem::undo(app, Some("撤销"))?)
                        .item(&PredefinedMenuItem::redo(app, Some("重做"))?)
                        .separator()
                        .item(&PredefinedMenuItem::cut(app, Some("剪切"))?)
                        .item(&PredefinedMenuItem::copy(app, Some("复制"))?)
                        .item(&PredefinedMenuItem::paste(app, Some("粘贴"))?)
                        .item(&PredefinedMenuItem::select_all(app, Some("全选"))?)
                        .build()?
                )
                .item(
                    &SubmenuBuilder::new(app, "窗口")
                        .item(&PredefinedMenuItem::minimize(app, Some("最小化"))?)
                        .item(&PredefinedMenuItem::close_window(app, Some("关闭窗口"))?)
                        .separator()
                        .item(&PredefinedMenuItem::fullscreen(app, Some("全屏"))?)
                        .build()?
                )
                .item(
                    &SubmenuBuilder::new(app, "帮助")
                        .item(&MenuItemBuilder::new("关于").id("about").build(app)?)
                        .build()?
                )
                .build()?;

            app.set_menu(menu)?;
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
            commands::trash::clear_expired_trash
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
