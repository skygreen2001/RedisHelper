# 优化"加载更多"用户体验

## 问题
当前点击"加载更多"后，请求瞬间完成，用户几乎感知不到任何变化：
- loading 图标一闪而过
- keys 列表直接追加，没有视觉反馈
- 数量文字更新不明显

## 方案

### 修改文件：`src/frontend/views/MainView.vue`

#### 1. 加载完成后自动滚动到新加载的 keys 位置
在 `handleLoadMore` 中，keys 追加后滚动 key-list 到新内容位置：

```typescript
// 在 keys.value = [...keys.value, ...nextBatch] 之后
// 滚动到新加载的第一条 key
await nextTick()
const treeEl = document.querySelector('.key-list-content .el-scrollbar__wrap')
if (treeEl) {
  // 滚动到倒数第 pageSize 个位置，让用户看到新内容
  const scrollTarget = treeEl.scrollHeight - treeEl.clientHeight - 50
  treeEl.scrollTo({ top: Math.max(0, scrollTarget), behavior: 'smooth' })
}
```

#### 2. 加载完成后短暂高亮数量文字
给 `count-divider` 添加一个临时高亮 class，300ms 后移除：

```typescript
// 在 handleLoadMore 成功后
const countEl = document.querySelector('.count-divider')
if (countEl) {
  countEl.classList.add('count-highlight')
  setTimeout(() => countEl.classList.remove('count-highlight'), 600)
}
```

对应 CSS：
```css
.count-divider {
  transition: color 0.3s ease, transform 0.3s ease;
}
.count-divider.count-highlight {
  color: #1890ff;
  transform: scale(1.15);
}
```

#### 3. 最小加载时间保证
确保 loading 状态至少显示 300ms，避免一闪而过：

```typescript
const startTime = Date.now()
// ... 加载逻辑 ...
const elapsed = Date.now() - startTime
if (elapsed < 300) {
  await new Promise(resolve => setTimeout(resolve, 300 - elapsed))
}
```

## 验证
1. 点击"加载更多"，应看到按钮 loading 至少 300ms
2. 加载完成后，数量文字短暂放大高亮
3. key 列表自动平滑滚动到新内容位置
