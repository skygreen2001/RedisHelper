#![cfg_attr(all(not(debug_assertions), target_os = "windows"), windows_subsystem = "windows")]

mod commands;
mod redis;
mod storage;

use tauri::Builder;

fn main() {
    Builder::default()
        .plugin(tauri_plugin_dialog::init())
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