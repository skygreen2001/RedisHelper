# 动态菜单文本切换实现计划

## 任务概述
优化 macOS 顶部菜单「窗口」子菜单中的「显示标签栏」菜单项，使其根据当前标签栏的显示状态动态切换文本：
- 当标签栏**显示**时，菜单显示「隐藏标签栏」
- 当标签栏**隐藏**时，菜单显示「显示标签栏」

## 当前状态分析

### 现有架构
1. **菜单定义**：在 `main.rs` 中使用 `MenuBuilder` 静态创建，菜单项文本固定为「显示标签栏」
2. **状态管理**：前端 `SessionManager` 管理每个窗口独立的 `showTabBar` 状态
3. **事件处理**：菜单点击通过 `menu-event` 事件传递到前端处理

### Tauri v2 能力
- `MenuItem::set_text()` 方法支持动态更新菜单项文本
- 支持窗口事件监听（包括焦点事件）
- 支持通过 Command 进行前后端通信

## 实现方案

### 第一阶段：Rust 后端改造

#### 1. 添加菜单状态管理结构
**文件**: `src-tauri/src/main.rs`

添加状态管理结构体用于存储每个窗口的菜单项引用：

```rust
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use tauri::menu::MenuItem;

// 菜单状态管理
struct MenuState {
    // window_label -> MenuItem 映射
    toggle_tab_bar_items: Arc<Mutex<HashMap<String, MenuItem>>>,
}

impl MenuState {
    fn new() -> Self {
        Self {
            toggle_tab_bar_items: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}
```

#### 2. 修改菜单创建逻辑
**文件**: `src-tauri/src/main.rs`

在 `setup` 函数中：
1. 创建 `MenuState` 并作为托管状态
2. 保存 `toggle_tab_bar` 菜单项到状态管理
3. 初始文本设为「隐藏标签栏」（因为默认显示标签栏）

#### 3. 添加更新菜单 Command
**文件**: `src-tauri/src/main.rs`

```rust
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
```

#### 4. 注册 Command
在 `tauri::Builder` 的 `.invoke_handler()` 中注册新 command。

---

### 第二阶段：前端状态同步

#### 1. 扩展 SessionManager
**文件**: `src/frontend/sessions/SessionManager.ts`

在 `showTabBar` 的 setter 中添加菜单同步逻辑：

```typescript
set showTabBar(value: boolean) {
    const oldValue = this.active.showTabBar;
    this.active.showTabBar = value;
    
    // 同步更新菜单文本
    if (oldValue !== value) {
        this.syncMenuText(value);
    }
}

private async syncMenuText(showTabBar: boolean) {
    if (!isTauriEnv()) return;
    try {
        const { invoke } = await import('@tauri-apps/api/core');
        await invoke('update_toggle_tab_bar_menu', { showTabBar });
    } catch (err) {
        console.error('同步菜单文本失败:', err);
    }
}
```

#### 2. 窗口焦点监听（可选增强）
**文件**: `src/frontend/views/MainView.vue`

添加窗口焦点监听，确保切换窗口时菜单状态正确：

```typescript
// 在 onMounted 中调用
const setupMenuSync = async () => {
    if (!isTauriEnv()) return;
    
    const { listen } = await import('@tauri-apps/api/event');
    const { getCurrentWindow } = await import('@tauri-apps/api/window');
    
    const currentWindow = getCurrentWindow();
    
    // 监听窗口焦点变化
    currentWindow.onFocusChanged(({ payload: focused }) => {
        if (focused) {
            // 窗口获得焦点时同步菜单状态
            const { invoke } = await import('@tauri-apps/api/core');
            invoke('update_toggle_tab_bar_menu', { 
                showTabBar: sessionManager.showTabBar 
            });
        }
    });
};
```

---

### 第三阶段：初始化同步

#### 1. 应用启动时同步
**文件**: `src/frontend/views/MainView.vue`

在应用启动时执行一次菜单状态同步：

```typescript
onMounted(async () => {
    // ... 现有代码
    
    // 初始化菜单文本
    if (isTauriEnv()) {
        const { invoke } = await import('@tauri-apps/api/core');
        await invoke('update_toggle_tab_bar_menu', { 
            showTabBar: sessionManager.showTabBar 
        });
    }
});
```

---

## 关键文件清单

| 文件路径 | 修改类型 | 说明 |
|---------|---------|------|
| `src-tauri/src/main.rs` | 修改 | 添加 MenuState 结构、update_toggle_tab_bar_menu command、修改菜单创建逻辑 |
| `src/frontend/sessions/SessionManager.ts` | 修改 | 在 showTabBar setter 中添加 syncMenuText 调用 |
| `src/frontend/views/MainView.vue` | 修改 | 添加初始化同步和窗口焦点监听 |

## 状态同步流程

```
用户点击菜单/快捷键切换标签栏
    ↓
前端: sessionManager.showTabBar = !sessionManager.showTabBar
    ↓
前端: syncMenuText() 调用 Rust command
    ↓
Rust: update_toggle_tab_bar_menu(showTabBar)
    ↓
Rust: menu_item.set_text(新文本)
    ↓
菜单显示更新后的文本
```

## 设计决策

1. **单一状态来源**：前端 `SessionManager` 是标签栏显示状态的唯一来源
2. **按需同步**：只在状态改变时同步，避免不必要的通信
3. **多窗口支持**：每个窗口有独立的菜单项存储，通过 window_label 区分
4. **向后兼容**：Rust 调用失败时前端静默处理，不影响核心功能

## 验证步骤

1. 启动应用，检查菜单是否显示「隐藏标签栏」（因为默认显示）
2. 点击菜单或按 Cmd+Shift+T，标签栏隐藏，菜单应变为「显示标签栏」
3. 再次点击菜单，标签栏显示，菜单应变为「隐藏标签栏」
4. 打开多个窗口，切换窗口焦点，检查每个窗口的菜单状态是否正确
