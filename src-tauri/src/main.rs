#![cfg_attr(all(not(debug_assertions), target_os = "windows"), windows_subsystem = "windows")]
#![allow(unexpected_cfgs)]

mod commands;
mod redis;
mod storage;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri::{Builder, Emitter, Manager, menu::{IconMenuItem, IconMenuItemBuilder, MenuBuilder, NativeIcon, PredefinedMenuItem, SubmenuBuilder, AboutMetadata}, Wry};

/// 菜单状态管理结构
struct MenuState {
    toggle_tab_bar_items: Arc<Mutex<HashMap<String, IconMenuItem<Wry>>>>,
}

impl MenuState {
    fn new() -> Self {
        Self {
            toggle_tab_bar_items: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

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

/// 更新标签栏菜单的文本和图标
#[tauri::command]
fn update_toggle_tab_bar_menu(
    window: tauri::Window,
    show_tab_bar: bool,
    state: tauri::State<MenuState>,
) -> Result<(), String> {
    let items = state.toggle_tab_bar_items.lock().unwrap();
    if let Some(menu_item) = items.get(window.label()) {
        // 更新文本
        let text = if show_tab_bar { "隐藏标签栏" } else { "显示标签栏" };
        menu_item.set_text(text).map_err(|e| e.to_string())?;
        
        // 更新图标：
        // 标签栏显示中（show_tab_bar=true）→ 文本"隐藏标签栏" → 空白占位（MenuMixedState）
        // 标签栏隐藏中（show_tab_bar=false）→ 文本"显示标签栏" → 勾选（MenuOnState）
        if show_tab_bar {
            menu_item.set_native_icon(Some(NativeIcon::MenuMixedState)).map_err(|e| e.to_string())?;
        } else {
            menu_item.set_native_icon(Some(NativeIcon::MenuOnState)).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

/// 移除 macOS 系统自动注入到 "编辑" 菜单中的系统项
/// (如 "开始听写/Start Dictation…"、"自动填充/Autofill"、"表情与符号/Emoji & Symbols" 等)
#[cfg(target_os = "macos")]
fn remove_macos_system_edit_menu_items() {
    use objc::{class, msg_send, sel, sel_impl};
    use objc::runtime::Object;
    use std::ffi::CStr;
    use std::ptr::null_mut;

    type ObjId = *mut Object;
    const NIL: ObjId = null_mut();

    unsafe {
        // 获取 NSApplication::sharedApplication()
        let app: ObjId = msg_send![class!(NSApplication), sharedApplication];
        if app == NIL {
            return;
        }

        // 获取 mainMenu
        let main_menu: ObjId = msg_send![app, mainMenu];
        if main_menu == NIL {
            return;
        }

        // 遍历主菜单项,找到 "编辑"/"Edit" 菜单
        let item_count: usize = msg_send![main_menu, numberOfItems];
        for i in 0..item_count {
            let item: ObjId = msg_send![main_menu, itemAtIndex: i as isize];
            if item == NIL {
                continue;
            }

            let title: ObjId = msg_send![item, title];
            if title == NIL {
                continue;
            }

            let title_utf8: *const u8 = msg_send![title, UTF8String];
            if title_utf8.is_null() {
                continue;
            }

            let title_str = CStr::from_ptr(title_utf8 as *const i8).to_string_lossy();
            if title_str != "编辑" && title_str != "Edit" {
                continue;
            }

            // 找到编辑菜单,获取其子菜单
            let submenu: ObjId = msg_send![item, submenu];
            if submenu == NIL {
                break;
            }

            // 收集需要移除的项的索引
            // 匹配系统自动注入的项(中英文均匹配)
            let sub_count: usize = msg_send![submenu, numberOfItems];
            let mut to_remove: Vec<usize> = Vec::new();

            for j in 0..sub_count {
                let sub_item: ObjId = msg_send![submenu, itemAtIndex: j as isize];
                if sub_item == NIL {
                    continue;
                }

                let item_title: ObjId = msg_send![sub_item, title];
                if item_title == NIL {
                    continue;
                }

                let item_utf8: *const u8 = msg_send![item_title, UTF8String];
                if item_utf8.is_null() {
                    continue;
                }

                let item_str = CStr::from_ptr(item_utf8 as *const i8).to_string_lossy();

                // 判断是否是系统自动添加的项
                let is_system_item = item_str.contains("Dictation")
                    || item_str.contains("听写")
                    || item_str.contains("AutoFill")
                    || item_str.contains("自动填充")
                    || item_str.contains("Substitutions")
                    || item_str.contains("替换")
                    || item_str.contains("Spelling")
                    || item_str.contains("拼写")
                    || item_str.contains("Transformations")
                    || item_str.contains("变换")
                    || item_str.contains("Speech")
                    || item_str.contains("语音");

                if is_system_item {
                    to_remove.push(j);
                }
            }

            // 从后往前移除,以保持索引正确
            for j in to_remove.iter().rev() {
                let _: () = msg_send![submenu, removeItemAtIndex: *j as isize];
            }

            // 清理尾部多余的分隔符(系统项移除后可能留下孤立分隔符)
            loop {
                let count: usize = msg_send![submenu, numberOfItems];
                if count == 0 {
                    break;
                }
                let last: ObjId = msg_send![submenu, itemAtIndex: (count - 1) as isize];
                if last == NIL {
                    break;
                }
                let is_separator: bool = msg_send![last, isSeparatorItem];
                if is_separator {
                    let _: () = msg_send![submenu, removeItemAtIndex: (count - 1) as isize];
                } else {
                    break;
                }
            }

            break;
        }
    }
}

fn main() {
    Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .manage(MenuState::new())
        .setup(|app| {
            // 创建中文菜单
            // toggle_tab_bar 使用 IconMenuItemBuilder（支持图标和动态更新文本）
            // toggle_tab_bar 初始状态：标签栏显示中 → 文本"隐藏标签栏" + 空白占位
            let toggle_tab_bar_item = IconMenuItemBuilder::with_id("toggle_tab_bar", "隐藏标签栏")
                .accelerator("CmdOrCtrl+Shift+T")
                .native_icon(NativeIcon::MenuMixedState)
                .build(app)?;

            // 带图标的菜单项使用 IconMenuItemBuilder
            let new_window_item = IconMenuItemBuilder::with_id("new_window", "新建窗口")
                .accelerator("CmdOrCtrl+N")
                .native_icon(NativeIcon::Add)
                .build(app)?;

            let help_doc_item = IconMenuItemBuilder::with_id("help_doc", "帮助文档")
                .native_icon(NativeIcon::Bookmarks)
                .build(app)?;

            let about_item = IconMenuItemBuilder::with_id("about", "关于")
                .native_icon(NativeIcon::Info)
                .build(app)?;

            // 保存菜单项到状态
            {
                let state = app.state::<MenuState>();
                let mut items = state.toggle_tab_bar_items.lock().unwrap();
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
                        .item(&new_window_item)
                        .separator()
                        .item(&PredefinedMenuItem::close_window(app, Some("关闭窗口"))?)
                        .separator()
                        .item(&toggle_tab_bar_item)
                        .separator()
                        .build()?
                )
                .item(
                    &SubmenuBuilder::new(app, "帮助")
                        .item(&help_doc_item)
                        .item(&about_item)
                        .build()?
                )
                .build()?;

            app.set_menu(menu)?;

            // macOS 会自动向 "编辑" 菜单注入系统项(开始听写、自动填充等),
            // 这里通过原生 API 移除它们,保持编辑菜单干净。
            #[cfg(target_os = "macos")]
            remove_macos_system_edit_menu_items();

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
            commands::server::save_server_order,
            commands::server::test_connection,
            commands::server::get_debug_log_enabled,
            commands::server::set_debug_log_enabled,
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
            commands::redis::slowlog_get,
            commands::redis::get_memory_info,
            commands::redis::get_type_distribution,
            commands::redis::get_server_info,
            commands::redis::get_key_stats,
            commands::export::export_data,
            commands::export::import_data,
            commands::trash::move_to_trash,
            commands::trash::batch_move_to_trash,
            commands::trash::get_trash_items,
            commands::trash::restore_from_trash,
            commands::trash::batch_restore_from_trash,
            commands::trash::permanent_delete_trash,
            commands::trash::clear_expired_trash,
            // 审计相关命令
            commands::audit::audit_get_logs,
            commands::audit::audit_get_stats,
            commands::audit::audit_clear,
            commands::audit::audit_get_config,
            commands::audit::audit_update_config,
            commands::audit::audit_generate_test_data,
            // 元素级操作命令（List/Set/ZSet/Hash）
            commands::redis::list_rpush,
            commands::redis::list_lset,
            commands::redis::list_lrem,
            commands::redis::set_sadd,
            commands::redis::set_srem,
            commands::redis::zset_zadd,
            commands::redis::zset_zrem,
            commands::redis::hash_hset,
            commands::redis::hash_hdel,
            create_new_window,
            update_toggle_tab_bar_menu
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
