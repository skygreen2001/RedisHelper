# 用户帮助文档编写计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 编写 Redis 小助手的用户帮助文档（HELP.md），包含 16 张功能截图

**Architecture:** 通过 Vite 开发服务器在浏览器中访问应用进行截图，截图保存到 help/images/，帮助文档保存到 help/HELP.md

**Tech Stack:** Markdown 文档 + browser_take_screenshot 截图工具

---

## Task 1: 创建目录结构

**Files:**
- Create: `help/` (目录)
- Create: `help/images/` (目录)

- [ ] **Step 1: 创建 help 和 help/images 目录**

```bash
mkdir -p help/images
```

---

## Task 2: 启动 Vite 开发服务器

**Files:**
- 无文件修改

- [ ] **Step 1: 启动 Vite 开发服务器（非阻塞）**

Run: `npm run dev`
Expected: Vite 开发服务器在 http://localhost:5173 启动

- [ ] **Step 2: 在浏览器中打开应用**

使用 browser_navigate 访问 http://localhost:5173

- [ ] **Step 3: 等待页面完全加载**

使用 browser_snapshot 确认页面已加载

---

## Task 3: 截取主界面和菜单栏截图（4 张）

**Files:**
- Create: `help/images/01-main-overview.png`
- Create: `help/images/02-menu-connection-dropdown.png`
- Create: `help/images/03-menu-db-dropdown.png`
- Create: `help/images/04-menu-more-dropdown.png`

- [ ] **Step 1: 截取主界面总览**

对整个应用窗口截图，保存为 `help/images/01-main-overview.png`

- [ ] **Step 2: 点击「连接」下拉并截图**

点击菜单栏「连接」按钮，展开下拉菜单后截图，保存为 `help/images/02-menu-connection-dropdown.png`

- [ ] **Step 3: 点击「DB」下拉并截图**

点击菜单栏「DB」按钮，展开下拉菜单后截图，保存为 `help/images/03-menu-db-dropdown.png`

- [ ] **Step 4: 点击「更多」下拉并截图**

点击菜单栏「更多」按钮，展开下拉菜单后截图，保存为 `help/images/04-menu-more-dropdown.png`

---

## Task 4: 截取搜索栏和键列表截图（2 张）

**Files:**
- Create: `help/images/05-search-bar.png`
- Create: `help/images/06-key-list.png`

- [ ] **Step 1: 截取搜索栏**

对搜索栏区域截图（刷新按钮、搜索框、搜索按钮、+添加按钮），保存为 `help/images/05-search-bar.png`

- [ ] **Step 2: 截取键列表和分页区域**

对键列表和底部加载区域截图，保存为 `help/images/06-key-list.png`

---

## Task 5: 截取值展示区截图（3 张）

**Files:**
- Create: `help/images/07-value-display-string.png`
- Create: `help/images/08-value-display-hash.png`
- Create: `help/images/09-value-display-other-types.png`

- [ ] **Step 1: 点击 STRING 类型键并截图**

在键列表中点击一个 STRING 类型的键，对右侧值展示区截图，保存为 `help/images/07-value-display-string.png`

- [ ] **Step 2: 点击 HASH 类型键并截图**

点击一个 HASH 类型的键，截图保存为 `help/images/08-value-display-hash.png`

- [ ] **Step 3: 点击 LIST/SET/ZSET 类型键并截图**

依次点击 LIST、SET、ZSET 类型的键，截图保存为 `help/images/09-value-display-other-types.png`

---

## Task 6: 截取添加键对话框截图（2 张）

**Files:**
- Create: `help/images/10-add-key-dialog.png`
- Create: `help/images/11-add-key-type-examples.png`

- [ ] **Step 1: 打开添加键对话框并截图**

点击搜索栏 + 按钮，截图保存为 `help/images/10-add-key-dialog.png`

- [ ] **Step 2: 切换不同类型并截图**

在对话框中切换 List/Set/ZSet/Hash 类型，展示对应占位符，截图保存为 `help/images/11-add-key-type-examples.png`

---

## Task 7: 截取服务器设置截图（2 张）

**Files:**
- Create: `help/images/12-server-config.png`
- Create: `help/images/13-add-server-dialog.png`

- [ ] **Step 1: 打开服务器设置并截图**

点击「连接」>「设置」，截图保存为 `help/images/12-server-config.png`

- [ ] **Step 2: 打开添加服务器对话框并截图**

点击「添加服务器」按钮，截图保存为 `help/images/13-add-server-dialog.png`

---

## Task 8: 截取多选删除和废键箱截图（2 张）

**Files:**
- Create: `help/images/14-multi-select-mode.png`
- Create: `help/images/15-trash-view.png`

- [ ] **Step 1: 进入多选模式并截图**

点击菜单栏「多选」按钮，勾选若干键，截图保存为 `help/images/14-multi-select-mode.png`

- [ ] **Step 2: 进入废键箱并截图**

点击「DB」>「废键箱」，截图保存为 `help/images/15-trash-view.png`

---

## Task 9: 截取导出对话框截图（1 张）

**Files:**
- Create: `help/images/16-export-dialog.png`

- [ ] **Step 1: 打开导出对话框并截图**

点击「更多」>「导出」，截图保存为 `help/images/16-export-dialog.png`

---

## Task 10: 编写 HELP.md 文档

**Files:**
- Create: `help/HELP.md`

- [ ] **Step 1: 编写完整的帮助文档**

文档结构如下：

```markdown
# Redis 小助手 - 用户帮助文档

## 一、应用简介
## 二、快速上手
  ### 2.1 安装与启动
  ### 2.2 添加服务器连接
  ### 2.3 连接服务器并选择数据库
## 三、主界面介绍
  ### 3.1 界面总览
  ### 3.2 菜单栏
  ### 3.3 搜索栏
  ### 3.4 状态栏
## 四、键值管理
  ### 4.1 浏览键列表
  ### 4.2 搜索键
  ### 4.3 查看键值（STRING/HASH/LIST/SET/ZSET）
  ### 4.4 添加键
  ### 4.5 编辑键值
  ### 4.6 删除键
## 五、多选删除
  ### 5.1 进入多选模式
  ### 5.2 选择键
  ### 5.3 全选操作
  ### 5.4 移入废键箱
  ### 5.5 快捷键
## 六、废键箱
  ### 6.1 进入废键箱
  ### 6.2 恢复键
  ### 6.3 永久删除
  ### 6.4 清理过期项
  ### 6.5 自动清理机制
## 七、数据导入导出
  ### 7.1 导出数据
  ### 7.2 导入数据
  ### 7.3 清空数据库
## 八、数据库管理
## 九、服务器管理
## 十、快捷键一览
## 十一、常见问题 (FAQ)
## 十二、注意事项
```

每节配合对应的截图引用：`![描述](images/xx-name.png)`

---

## Task 11: 审查和验证

- [ ] **Step 1: 检查所有截图文件存在**

确认 help/images/ 下有 16 张截图

- [ ] **Step 2: 检查文档中截图引用路径正确**

确认 HELP.md 中所有图片引用与实际文件名一致

- [ ] **Step 3: 检查文档结构完整性**

确认所有功能点都有覆盖

---

## 注意事项

1. **截图方式**：使用 `npm run dev`（Vite 开发服务器）在浏览器中截图，而非 `npm run tauri:dev`（原生窗口）
2. **Redis 依赖**：截图需要真实 Redis 连接和测试数据
3. **Tauri API 限制**：浏览器环境下部分 Tauri API（文件对话框等）不可用，相关截图可能需要跳过或使用替代方案
4. **截图质量**：确保截图清晰、完整，包含必要的 UI 元素
