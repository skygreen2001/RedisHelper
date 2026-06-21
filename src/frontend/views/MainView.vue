<template>
  <div class="main-container">
    <!-- 全局加载遮罩（覆盖整个APP） -->
    <Transition name="fade">
      <div v-if="isGlobalLoading" class="global-loading-overlay-full">
        <div class="loading-card">
          <el-icon class="loading-spinner-large" :size="48"><Loading /></el-icon>
          <div class="loading-title">{{ loadingMessage }}</div>
          <div class="loading-tip">请稍候...</div>
          <el-button 
            type="danger" 
            size="large" 
            @click="cancelConnectionSwitch"
            class="cancel-btn"
          >
            取消连接
          </el-button>
        </div>
      </div>
    </Transition>

    <!-- 标签栏（最顶部） -->
    <TabBar @new-tab="handleNewTabFromTabBar" />

    <!-- 消息提示 -->
    <el-alert
      v-if="message"
      :title="message"
      :type="messageType"
      show-icon
      :closable="true"
      @close="sessionManager.active.message = ''"
      class="message-alert"
    />

    <!-- 菜单栏 -->
    <div class="menu-bar">
      <div class="menu-left">
        <el-dropdown ref="serverDropdownRef" @command="handleDeviceCommand">
          <span class="el-dropdown-link">
            {{ selectedServer?.name || '连接' }} <el-icon class="el-icon--right"><ArrowDown /></el-icon>
          </span>
          <template #dropdown>
            <el-dropdown-menu>
              <el-dropdown-item
                v-for="server in servers"
                :key="server.id"
                :command="{ type: 'select', server }"
              >
                {{ server.name }}
              </el-dropdown-item>
              <el-dropdown-item divided command="settings">
                <el-icon><Setting /></el-icon> 设置
              </el-dropdown-item>
            </el-dropdown-menu>
          </template>
        </el-dropdown>
        <el-dropdown @command="handleDbCommand">
          <span class="el-dropdown-link">
            DB{{ selectedDb !== null ? ' ' + selectedDb : '' }} <el-icon class="el-icon--right"><ArrowDown /></el-icon>
          </span>
          <template #dropdown>
            <el-dropdown-menu>
              <el-dropdown-item
                v-for="db in databases"
                :key="db[0]"
                :command="{ type: 'select', db: db[0] }"
              >
                DB {{ db[0] }} ({{ db[1] }} 个)
              </el-dropdown-item>
              <el-dropdown-item divided command="trash">
                <el-icon><FolderOpened /></el-icon> 废键箱
                <el-badge v-if="currentServerTrashCount > 0" :value="currentServerTrashCount" class="menu-badge" />
              </el-dropdown-item>
              <el-dropdown-item divided command="add">
                <el-icon><Plus /></el-icon> 新增DB
              </el-dropdown-item>
              <el-dropdown-item divided command="delete" :disabled="isCurrentServerReadonly">
                <el-icon><Delete /></el-icon> 删除DB
                <span v-if="isCurrentServerReadonly" class="menu-disabled-hint">（只读模式）</span>
              </el-dropdown-item>
            </el-dropdown-menu>
          </template>
        </el-dropdown>
        <el-dropdown v-if="!isTrashView" @command="handleActionCommand">
          <span class="el-dropdown-link">
            更多 <el-icon class="el-icon--right"><ArrowDown /></el-icon>
          </span>
          <template #dropdown>
            <el-dropdown-menu>
              <el-dropdown-item command="import">
                <el-icon><Upload /></el-icon> 导入
              </el-dropdown-item>
              <el-dropdown-item command="export">
                <el-icon><Download /></el-icon> 导出
              </el-dropdown-item>
              <el-dropdown-item command="flush" divided :disabled="isCurrentServerReadonly">
                <el-icon><Delete /></el-icon> 清空
                <span v-if="isCurrentServerReadonly" class="menu-disabled-hint">（只读模式）</span>
              </el-dropdown-item>
              <el-dropdown-item command="generateTestData">
                <el-icon><Plus /></el-icon> 生成数据
              </el-dropdown-item>
              <el-dropdown-item command="memory" divided>
                <el-icon><DataAnalysis /></el-icon> 内存分析
              </el-dropdown-item>
              <el-dropdown-item command="log" divided>
                <el-icon><Document /></el-icon> 日志
              </el-dropdown-item>
            </el-dropdown-menu>
          </template>
        </el-dropdown>
        <el-button
          v-if="!isTrashView"
          :type="isMultiSelectMode ? 'warning' : 'default'"
          size="small"
          class="menu-multi-select-btn"
          @click="toggleMultiSelectMode"
        >
          <el-icon><Select /></el-icon>
          <span>{{ isMultiSelectMode ? '取消' : '多选' }}</span>
          <el-tag v-if="isMultiSelectMode && selectedKeys.length > 0"
                  type="danger" size="small" class="multi-select-count-tag">
            {{ selectedKeys.length }}
          </el-tag>
        </el-button>
        <el-dropdown v-if="!isTrashView" @command="handleSortCommand">
          <span class="el-dropdown-link sort-link">
            排序 <el-icon class="el-icon--right"><ArrowDown /></el-icon>
          </span>
          <template #dropdown>
            <el-dropdown-menu>
              <el-dropdown-item command="asc">
                <el-icon><SortUp /></el-icon> 升序 A→Z
              </el-dropdown-item>
              <el-dropdown-item command="desc">
                <el-icon><SortDown /></el-icon> 降序 Z→A
              </el-dropdown-item>
            </el-dropdown-menu>
          </template>
        </el-dropdown>
      </div>
    </div>

    <!-- 状态栏 -->
    <div class="status-bar">
      <template v-if="!isTrashView">
        当前服务器: {{ selectedServer?.name || '未选择' }} > DB: {{ selectedDb !== null ? `db${selectedDb}` : '未选择' }} {{ databases.find(db => db[0] === selectedDb)?.[1] ? `(共计${databases.find(db => db[0] === selectedDb)?.[1]}个)` : '' }} > key: {{ selectedKey || '未选择' }}
      </template>
      <template v-else>
        废键箱 - {{ selectedServer?.name || selectedServer?.host }}:{{ selectedServer?.port }}
      </template>
      <span v-if="isDebugMode" class="debug-info"> | {{ isRunningInTauri ? '✓ Tauri环境' : '✗ 浏览器环境' }}</span>
    </div>

    <!-- 键列表和值展示 -->
    <div class="content-area" ref="contentAreaRef">
      <template v-if="!isTrashView">
      <!-- 左侧键列表区 -->
      <div class="key-list" :style="{ width: keyListWidth + 'px' }">
        <!-- 搜索和操作栏 -->
        <div class="search-and-actions">
          <!-- 搜索模式 -->
          <div class="search-section">
            <el-button
              type="primary"
              plain
              size="small"
              @click="handleRefresh"
              class="refresh-btn"
            >
              <el-icon><Refresh /></el-icon>
            </el-button>
            <el-input
              v-model="sessionManager.active.searchPattern"
              :placeholder="searchPlaceholder"
              clearable
              @keyup.enter="searchKeys"
              @clear="sessionManager.active.searchPattern = ''; sessionManager.active.message = ''"
              class="search-input"
            />
            <el-dropdown @command="handleSearchOptionCommand" class="search-options-dropdown">
              <el-icon :class="{ 'has-active-option': caseSensitive || !searchAll }"><Setting /></el-icon>
              <template #dropdown>
                <el-dropdown-menu>
                  <el-dropdown-item command="toggleCaseSensitive">
                    <el-icon v-if="!caseSensitive"><Check /></el-icon>
                    <span v-else style="display:inline-block;width:16px;"></span>
                    不区分大小写
                  </el-dropdown-item>
                  <el-dropdown-item command="toggleSearchAll">
                    <el-icon v-if="searchAll"><Check /></el-icon>
                    <span v-else style="display:inline-block;width:16px;"></span>
                    搜索全部
                  </el-dropdown-item>
                </el-dropdown-menu>
              </template>
            </el-dropdown>
            <el-button
              type="primary"
              size="small"
              @click="searchKeys"
              class="search-btn"
            >
              搜索
            </el-button>
            <el-tooltip content="添加键" placement="top">
              <el-button
                type="success"
                size="small"
                @click="sessionManager.active.showAddKeyDialog = true"
                class="add-key-btn"
              >
                <el-icon><Plus /></el-icon>
              </el-button>
            </el-tooltip>
          </div>
        </div>
        <!-- 键列表 -->
        <div class="key-list-content">
          <el-scrollbar>
            <el-tree
              :data="keyTree"
              node-key="id"
              default-expand-all
              :show-checkbox="isMultiSelectMode"
              :highlight-current="!isMultiSelectMode"
              @node-click="handleKeyClick"
              @check-change="handleCheckChange"
              ref="treeRef"
              empty-text="暂无数据"
            >
              <template #default="{ node }">
                <span class="key-item">{{ node.label }}</span>
              </template>
            </el-tree>
          </el-scrollbar>
        </div>

        <!-- 分页加载控制区域：全部加载后完全隐藏，让出空间 -->
        <div class="key-list-footer" v-if="showKeyListFooter">

          <!-- 多选操作面板（向上展开） -->
          <el-collapse-transition>
            <div class="multi-select-panel" v-show="isMultiSelectMode">
              <div class="multi-select-panel-actions">
                <el-dropdown @command="handleSelectAllCommand" trigger="click">
                  <el-button size="small">
                    全选 <el-icon class="el-icon--right"><ArrowDown /></el-icon>
                  </el-button>
                  <template #dropdown>
                    <el-dropdown-menu>
                      <el-dropdown-item command="loaded">
                        全选当前已加载 ({{ loadedCount }})
                      </el-dropdown-item>
                      <el-dropdown-item command="all">
                        全选所有 ({{ keysTotal }})
                      </el-dropdown-item>
                    </el-dropdown-menu>
                  </template>
                </el-dropdown>
                <el-button size="small" @click="clearSelection" :disabled="selectedKeys.length === 0">
                  取消
                </el-button>
                <el-button
                  type="danger"
                  size="small"
                  @click="batchMoveToTrash"
                  :disabled="selectedKeys.length === 0 || isCurrentServerReadonly"
                >
                  <el-icon><Delete /></el-icon>
                  删除 ({{ selectedKeys.length }})
                </el-button>
              </div>
              <div class="multi-select-panel-hint">
                快捷键: Ctrl+A 全选当前 | Ctrl+Shift+A 全选所有 | Esc 退出
              </div>
            </div>
          </el-collapse-transition>

          <!-- 加载所有进度显示 -->
          <div class="loading-all-progress" v-if="isLoadingAll">
            <el-progress
              :percentage="keysTotal > 0 ? Math.round((loadedCount / keysTotal) * 100) : 0"
              :show-text="true"
              :stroke-width="10"
              striped
              striped-flow
            />
            <span class="loading-text">正在加载所有 keys... ({{ loadedCount }} / {{ keysTotal }})</span>
          </div>

          <!-- 搜索全部模式：仅显示匹配数量 -->
          <div class="load-actions" v-if="searchPattern.trim() && searchAll">
            <span class="search-match-count">
              匹配 {{ filteredKeys.length }} 个
            </span>
          </div>

          <!-- 非搜索全部模式：加载按钮 + 数量 + 搜索匹配数量 -->
          <div class="load-actions" v-else-if="(hasMoreKeys || isLoadingAll) && keysTotal > 0">
            <el-button
              class="load-btn"
              @click="handleLoadMore"
              :loading="isLoadingMore"
              :disabled="isLoadingAll || !hasMoreKeys"
            >
              加载更多
            </el-button>
            <span class="count-divider">
              <template v-if="keysTotal > 0">{{ loadedCount }} / {{ keysTotal }}</template>
              <template v-else>{{ loadedCount }} 个 keys</template>
            </span>
            <el-button
              class="load-btn"
              @click="handleLoadAll"
              :disabled="isLoadingMore || !hasMoreKeys"
            >
              加载所有
            </el-button>
            <!-- 搜索匹配数量（仅已加载模式） -->
            <span v-if="searchPattern.trim() && filteredKeys.length > 0" class="search-match-count">
              匹配 {{ filteredKeys.length }} 个
            </span>
          </div>
        </div>
      </div>

      <!-- 可拖拽分隔条 -->
      <div 
        class="resize-divider"
        @mousedown="startResize"
        :class="{ 'resizing': isResizing }"
      >
        <div class="resize-handle"></div>
      </div>

      <!-- 右侧值展示区 -->
      <div class="value-display" style="position: relative;">
        <div v-if="selectedKey" class="key-detail">
          <div class="value-header">
            <div class="value-type">类型: {{ keyType.toUpperCase() }}</div>
            <!-- 复杂类型切换编辑模式 -->
            <div v-if="isComplexType" class="edit-mode-switch">
              <el-radio-group v-model="sessionManager.active.editMode" size="small">
                <el-radio-button value="overwrite">覆盖模式</el-radio-button>
                <el-radio-button value="element">元素编辑</el-radio-button>
              </el-radio-group>
            </div>
          </div>
          <div class="value-content">
            <!-- 覆盖模式：原有 textarea -->
            <el-input
              v-if="!isComplexType || sessionManager.active.editMode === 'overwrite'"
              v-model="sessionManager.active.editKeyForm.value"
              type="textarea"
              :rows="10"
              :placeholder="isComplexType ? typePlaceholders[keyType] || '请输入 JSON 格式的值' : '请输入值'"
              class="value-editor"
            />
            <!-- 元素编辑模式：Table 列表 -->
            <div v-else class="element-editor">
              <!-- 工具栏 -->
              <div class="element-toolbar">
                <el-button type="primary" size="small" @click="openAddElementDialog" :disabled="isCurrentServerReadonly">
                  <el-icon><Plus /></el-icon>添加元素
                </el-button>
                <span class="element-count">ID (Total: {{ parsedElements.length }})</span>
                <!-- ZSet 排序选择器 -->
                <el-select
                  v-if="keyType === 'zset'"
                  v-model="zsetSortOrder"
                  size="small"
                  class="zset-sort-select"
                  placeholder="排序方式"
                >
                  <el-option label="默认顺序" value="default" />
                  <el-option label="Score 升序" value="score-asc" />
                  <el-option label="Score 降序" value="score-desc" />
                  <el-option label="Member A-Z" value="member-asc" />
                  <el-option label="Member Z-A" value="member-desc" />
                </el-select>
                <el-input
                  v-model="elementSearchKeyword"
                  placeholder="输入关键字搜索"
                  size="small"
                  clearable
                  class="element-search"
                />
              </div>
              <!-- 元素表格 -->
              <el-table
                :data="filteredElements"
                border
                stripe
                size="small"
                class="element-table"
                max-height="400"
                empty-text="暂无数据"
              >
                <!-- 索引列 -->
                <el-table-column label="ID" width="80" align="center">
                  <template #default="{ $index }">
                    {{ $index + 1 }}
                  </template>
                </el-table-column>
                <!-- ZSet: Member 列 + Score 列 -->
                <template v-if="keyType === 'zset'">
                  <el-table-column label="Member" min-width="200" show-overflow-tooltip>
                    <template #default="{ row, $index }">
                      <div v-if="editingIndex === $index" class="inline-edit">
                        <el-input
                          v-model="editBuffer.value"
                          size="small"
                          placeholder="Member"
                          class="value-input"
                        />
                        <el-input
                          v-model.number="editBuffer.score"
                          size="small"
                          placeholder="Score"
                          type="number"
                          class="score-input"
                        />
                      </div>
                      <span v-else>{{ row.value }}</span>
                    </template>
                  </el-table-column>
                  <el-table-column label="Score" width="120" align="right">
                    <template #default="{ row, $index }">
                      <span v-if="editingIndex !== $index">{{ row.score }}</span>
                    </template>
                  </el-table-column>
                </template>
                <!-- 其他类型: Value 列 -->
                <el-table-column v-else label="Value" min-width="200" show-overflow-tooltip>
                  <template #default="{ row, $index }">
                    <div v-if="editingIndex === $index" class="inline-edit">
                      <el-input
                        v-if="keyType === 'hash'"
                        v-model="editBuffer.field"
                        size="small"
                        placeholder="Field"
                        class="field-input"
                      />
                      <el-input
                        v-model="editBuffer.value"
                        size="small"
                        placeholder="Value"
                        class="value-input"
                      />
                    </div>
                    <span v-else>{{ formatElementDisplay(row) }}</span>
                  </template>
                </el-table-column>
                <!-- 操作列 -->
                <el-table-column label="操作" width="180" align="center" fixed="right">
                  <template #default="{ row, $index }">
                    <div v-if="editingIndex === $index" class="action-group">
                      <el-button type="success" link size="small" @click="saveElement($index)">
                        <el-icon><Check /></el-icon>
                      </el-button>
                      <el-button link size="small" @click="cancelEdit">
                        <el-icon><Close /></el-icon>
                      </el-button>
                    </div>
                    <div v-else class="action-group">
                      <el-button link size="small" @click="viewElement(row)" title="查看详情">
                        <el-icon><Document /></el-icon>
                      </el-button>
                      <el-button link size="small" @click="startEdit($index)" :disabled="isCurrentServerReadonly" title="编辑">
                        <el-icon><Edit /></el-icon>
                      </el-button>
                      <el-button link size="small" type="danger" @click="removeElement($index)" :disabled="isCurrentServerReadonly" title="删除">
                        <el-icon><Delete /></el-icon>
                      </el-button>
                    </div>
                  </template>
                </el-table-column>
              </el-table>
            </div>
          </div>
          <div class="value-actions">
            <el-button size="small" type="danger" @click="deleteKey" class="action-btn delete-btn" :disabled="isCurrentServerReadonly">
              <el-icon><Delete /></el-icon>
              <span>删除</span>
            </el-button>
            <el-button size="small" type="primary" @click="updateKey" class="action-btn edit-btn">
              <el-icon><Edit /></el-icon>
              <span>修改</span>
            </el-button>
            <el-button size="small" @click="refreshKeyValue" class="action-btn refresh-btn">
              <el-icon><Refresh /></el-icon>
              <span>刷新</span>
            </el-button>
          </div>
        </div>
        <div v-else class="home-view-wrapper">
          <HomeView :server="selectedServer" :db="selectedDb" />
        </div>
      </div>
      </template>
      <template v-else>
        <div class="trash-view">
          <div class="trash-toolbar">
            <el-button type="primary" size="small" @click="loadTrashItems" :loading="trash.isLoading">
              <el-icon><Refresh /></el-icon> 刷新
            </el-button>
            <el-button v-if="trashSelectedIds.length > 0" type="success" size="small" @click="batchRestoreFromTrash">
              <el-icon><Refresh /></el-icon> 恢复选中 ({{ trashSelectedIds.length }})
            </el-button>
            <el-button v-if="trashSelectedIds.length > 0" type="danger" size="small" @click="permanentDeleteTrash">
              <el-icon><Delete /></el-icon> 永久删除选中 ({{ trashSelectedIds.length }})
            </el-button>
            <el-button type="info" size="small" @click="clearExpiredTrash">清理过期项</el-button>
          </div>
          <div class="trash-list-content">
            <el-table :data="trash.trashItems" @selection-change="handleTrashSelectionChange" empty-text="废键箱为空" stripe style="width:100%">
              <el-table-column type="selection" width="40" />
              <el-table-column prop="key" label="键名" min-width="200" show-overflow-tooltip />
              <el-table-column prop="key_type" label="类型" width="80">
                <template #default="{ row }">
                  <el-tag size="small" :type="getTypeTagColor(row.key_type)">{{ row.key_type.toUpperCase() }}</el-tag>
                </template>
              </el-table-column>
              <el-table-column prop="host" label="服务器" width="140" show-overflow-tooltip />
              <el-table-column prop="db" label="DB" width="60" />
              <el-table-column prop="deleted_at" label="删除时间" width="180" show-overflow-tooltip />
              <el-table-column prop="expires_at" label="过期时间" width="180" show-overflow-tooltip />
              <el-table-column label="操作" width="120" fixed="right">
                <template #default="{ row }">
                  <el-button type="primary" link size="small" @click="restoreSingleItem(row.id)">恢复</el-button>
                  <el-button type="danger" link size="small" @click="deleteSingleItem(row.id)">删除</el-button>
                </template>
              </el-table-column>
            </el-table>
          </div>
        </div>
      </template>
    </div>

    <!-- 底部标题栏 -->
    <div class="footer">
      Redis 系统数据监控 只为更好的体验
    </div>

    <!-- 添加键对话框 -->
    <el-dialog
      v-model="sessionManager.active.showAddKeyDialog"
      title="添加键"
      width="500px"
      custom-class="bounce-dialog"
    >
      <el-form :model="newKeyForm" label-width="80px">
        <el-form-item label="键名" required>
          <el-input v-model="sessionManager.active.newKeyForm.key" placeholder="输入键名" />
        </el-form-item>
        <el-form-item label="类型" required>
          <el-radio-group v-model="sessionManager.active.newKeyForm.type" size="default">
            <el-radio-button label="string">String</el-radio-button>
            <el-radio-button label="list">List</el-radio-button>
            <el-radio-button label="set">Set</el-radio-button>
            <el-radio-button label="zset">ZSet</el-radio-button>
            <el-radio-button label="hash">Hash</el-radio-button>
          </el-radio-group>
        </el-form-item>
        <el-form-item label="值" required>
          <el-input
            v-model="sessionManager.active.newKeyForm.value"
            type="textarea"
            :rows="4"
            :placeholder="typePlaceholders[newKeyForm.type]"
          />
          <div class="type-hint">
            <span class="hint-label">示例：</span>
            <span class="hint-text">{{ typeExamples[newKeyForm.type] }}</span>
          </div>
        </el-form-item>
      </el-form>
      <template #footer>
        <span class="dialog-footer">
          <el-button @click="sessionManager.active.showAddKeyDialog = false">取消</el-button>
          <el-button type="primary" @click="addKey">确定</el-button>
        </span>
      </template>
    </el-dialog>

    <!-- 编辑键对话框 -->
    <el-dialog
      v-model="sessionManager.active.showEditKeyDialog"
      title="修改键"
      width="500px"
    >
      <el-form :model="editKeyForm" label-width="80px">
        <el-form-item label="键名" required>
          <el-input v-model="sessionManager.active.editKeyForm.key" disabled />
        </el-form-item>
        <el-form-item label="值" required>
          <el-input
            v-model="sessionManager.active.editKeyForm.value"
            type="textarea"
            :rows="4"
            placeholder="输入值"
          />
        </el-form-item>
        <el-form-item label="类型" required>
          <el-select v-model="sessionManager.active.editKeyForm.type" placeholder="选择类型">
            <el-option label="String" value="string" />
            <el-option label="List" value="list" />
            <el-option label="Set" value="set" />
            <el-option label="ZSet" value="zset" />
            <el-option label="Hash" value="hash" />
          </el-select>
        </el-form-item>
      </el-form>
      <template #footer>
        <span class="dialog-footer">
          <el-button @click="sessionManager.active.showEditKeyDialog = false">取消</el-button>
          <el-button type="primary" @click="updateKey">确定</el-button>
        </span>
      </template>
    </el-dialog>

    <!-- 添加元素对话框（元素编辑模式） -->
    <el-dialog
      v-model="showAddElementDialog"
      :title="addElementDialogTitle"
      width="480px"
      align-center
      :close-on-click-modal="false"
    >
      <el-form :model="addElementForm" label-width="90px" @submit.enter.prevent="confirmAddElement">
        <!-- Hash 类型：Field + Value -->
        <template v-if="keyType === 'hash'">
          <el-form-item label="Field" required>
            <el-input
              v-model="addElementForm.field"
              placeholder="请输入 Field 名称"
              autofocus
            />
          </el-form-item>
          <el-form-item label="Value" required>
            <el-input
              v-model="addElementForm.value"
              type="textarea"
              :rows="3"
              placeholder="请输入 Value"
            />
          </el-form-item>
        </template>
        <!-- ZSet 类型：Member + Score -->
        <template v-else-if="keyType === 'zset'">
          <el-form-item label="Member" required>
            <el-input
              v-model="addElementForm.value"
              placeholder="请输入成员名称"
              autofocus
            />
          </el-form-item>
          <el-form-item label="Score" required>
            <el-input
              v-model="addElementForm.score"
              type="number"
              placeholder="请输入分数（数字）"
            />
          </el-form-item>
        </template>
        <!-- List / Set 类型：Value -->
        <template v-else>
          <el-form-item label="Value" required>
            <el-input
              v-model="addElementForm.value"
              type="textarea"
              :rows="4"
              placeholder="请输入元素值"
              autofocus
            />
          </el-form-item>
        </template>
      </el-form>
      <template #footer>
        <span class="dialog-footer">
          <el-button @click="showAddElementDialog = false">取消</el-button>
          <el-button type="primary" :loading="isAddingElement" @click="confirmAddElement">确定添加</el-button>
        </span>
      </template>
    </el-dialog>

    <!-- 服务器配置页面 -->
    <el-dialog
      v-model="sessionManager.active.showServerConfig"
      title="服务器配置"
      width="800px"
      class="server-config-dialog"
      align-center
      destroy-on-close
    >
      <ServerConfigView />
      <template #footer>
        <span class="dialog-footer">
          <el-button @click="closeServerConfig">关闭</el-button>
        </span>
      </template>
    </el-dialog>

    <!-- 选择连接对话框 -->
    <el-dialog
      v-model="showSelectServerDialog"
      title="选择连接"
      width="450px"
      :close-on-click-modal="false"
      :show-close="sessionManager.sessions.length > 1"
      :close-on-press-escape="sessionManager.sessions.length > 1"
      align-center
      @close="handleCancelNewTab"
    >
      <p class="select-server-hint">请选择一个服务器连接以创建标签页</p>
      <div class="select-server-list">
        <div
          v-for="server in servers"
          :key="server.id"
          class="select-server-item"
          @click="handleNewTabSelectServer(server)"
        >
          <div class="server-item-info">
            <span class="server-item-name">{{ server.name }}</span>
            <span class="server-item-addr">{{ server.host }}:{{ server.port }}</span>
          </div>
        </div>
        <div v-if="servers.length === 0" class="select-server-empty">
          <p>暂无可用连接</p>
          <el-button type="primary" size="small" @click="openServerConfigFromDialog">前往设置</el-button>
        </div>
      </div>
      <template #footer>
        <span class="dialog-footer">
          <el-button
            v-if="sessionManager.sessions.length > 1"
            @click="handleCancelNewTab"
          >
            取消
          </el-button>
          <span v-else></span>
        </span>
      </template>
    </el-dialog>

    <!-- 新增DB对话框 -->
    <el-dialog
      v-model="sessionManager.active.showAddDbDialog"
      title="新增DB"
      width="400px"
    >
      <el-form :model="{ db: newDbNumber }" label-width="80px">
        <el-form-item label="DB编号" required>
          <el-input-number v-model="sessionManager.active.newDbNumber" :min="0" :max="15" />
        </el-form-item>
      </el-form>
      <template #footer>
        <span class="dialog-footer">
          <el-button @click="sessionManager.active.showAddDbDialog = false">取消</el-button>
          <el-button type="primary" @click="addDb">确定</el-button>
        </span>
      </template>
    </el-dialog>

    <!-- 删除DB对话框 -->
    <el-dialog
      v-model="sessionManager.active.showDeleteDbDialog"
      title="删除DB"
      width="400px"
    >
      <div class="delete-db-content">
        <p>请选择要删除的DB：</p>
        <div class="db-tags">
          <el-tag
            v-for="db in databases"
            :key="db[0]"
            :type="selectedDbsForDelete.includes(db[0]) ? 'danger' : ''"
            @click="toggleDbSelection(db[0])"
            class="db-tag"
          >
            DB {{ db[0] }} ({{ db[1] }} 个)
          </el-tag>
        </div>
      </div>
      <template #footer>
        <span class="dialog-footer">
          <el-button @click="sessionManager.active.showDeleteDbDialog = false">取消</el-button>
          <el-button type="danger" @click="deleteDb" :disabled="selectedDbsForDelete.length === 0">
            删除
          </el-button>
        </span>
      </template>
    </el-dialog>

    <!-- 导出对话框 -->
    <el-dialog
      v-model="sessionManager.active.showExportDialog"
      title="导出数据"
      width="480px"
    >
      <div class="export-content">
        <el-form label-width="80px">
          <el-form-item label="保存位置">
            <div class="folder-select-row">
              <el-input
                v-model="sessionManager.active.exportFolderPath"
                placeholder="点击右侧按钮选择文件夹"
                readonly
                class="folder-path-input"
              />
              <el-button 
                type="primary" 
                @click="selectExportFolder"
                :loading="isFolderLoading"
                class="folder-select-btn"
              >
                {{ isFolderLoading ? '加载中...' : '选择文件夹' }}
              </el-button>
            </div>
          </el-form-item>
          <el-form-item label="文件名">
            <el-input
              v-model="sessionManager.active.exportFileName"
              placeholder="请输入导出文件名（不含扩展名）"
            />
          </el-form-item>
        </el-form>
      </div>
      <template #footer>
        <span class="dialog-footer">
          <el-button @click="sessionManager.active.showExportDialog = false">取消</el-button>
          <el-button type="primary" @click="handleExport" :disabled="!exportFolderPath">
            导出
          </el-button>
        </span>
      </template>
    </el-dialog>
    
    <!-- 清空确认对话框 -->
    <el-dialog
      v-model="sessionManager.active.showFlushDialog"      title="确认清空"
      width="400px"
    >
      <div class="flush-confirm-content">
        <p>确定要清空当前数据库中的所有数据吗？</p>
        <p class="flush-warning">此操作不可恢复！</p>
      </div>
      <template #footer>
        <span class="dialog-footer">
          <el-button @click="sessionManager.active.showFlushDialog = false">取消</el-button>
          <el-button type="danger" @click="handleFlush">
            确认清空
          </el-button>
        </span>
      </template>
    </el-dialog>

    <!-- 日志对话框 -->
    <LogDialog 
      v-model="showLogDialog" 
      :host="selectedServer?.host" 
      :port="selectedServer?.port" 
      :password="selectedServer?.password" 
    />
    
    <!-- 内存分析对话框 -->
    <MemoryDialog 
      v-model="showMemoryDialog" 
      ref="memoryDialogRef"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, nextTick, watch } from 'vue'
import { Plus, Delete, Edit, ArrowDown, Setting, Refresh, FolderOpened, Select, Upload, Download, SortUp, SortDown, Check, Document, DataAnalysis, Loading, Close } from '@element-plus/icons-vue'
import { serverStore } from '../stores/serverStore'
import { redisStore } from '../stores/redisStore'
import { trashStore } from '../stores/trashStore'
import { ElMessageBox, ElMessage } from 'element-plus'
import ServerConfigView from './ServerConfigView.vue'
import HomeView from './HomeView.vue'
import TabBar from '../components/TabBar.vue'
import LogDialog from '../components/LogDialog.vue'
import MemoryDialog from '../components/MemoryDialog.vue'
import { sessionManager } from '../sessions/SessionManager'
import { isTauriEnv } from '../utils/tauri'
import { useLogStore } from '../stores/logStore'

const logStore = useLogStore()
const showLogDialog = ref(false)
const showMemoryDialog = ref(false)
const memoryDialogRef = ref<InstanceType<typeof MemoryDialog> | null>(null)
const serverDropdownRef = ref<any>(null)

// 简化的 Tauri 环境检测
function checkIsTauri(): boolean {
  try {
    if (typeof window === 'undefined') return false
    const win = window as any
    // Tauri v2 使用 __TAURI_INTERNALS__ 而非 __TAURI__
    return !!(win.__TAURI__ || win.__TAURI_INTERNALS__ || win.__TAURI_IPC__)
  } catch (e) {
    console.warn('Tauri检测失败:', e)
    return false
  }
}

// 调试模式 - 开发环境自动开启，生产构建自动关闭
const isDebugMode = import.meta.env.DEV
const isRunningInTauri = ref(checkIsTauri())

// Tauri 窗口加载后 __TAURI__ 对象才注入，需要轮询检测
if (!isRunningInTauri.value) {
  const timer = setInterval(() => {
    if (checkIsTauri()) {
      isRunningInTauri.value = true
      clearInterval(timer)
    }
  }, 500)
  // 最多检测 10 秒
  setTimeout(() => clearInterval(timer), 10000)
}

const server = serverStore()
const redis = redisStore()
const trash = trashStore()

// ========== 分隔条拖拽相关 ==========
const contentAreaRef = ref<HTMLElement | null>(null)
const KEY_LIST_WIDTH_KEY = 'redis-helper-key-list-width'
const MIN_KEY_LIST_WIDTH = 200
const MAX_KEY_LIST_WIDTH_RATIO = 0.8

const keyListWidth = ref(400)
const isResizing = ref(false)

const startResize = (e: MouseEvent) => {
  e.preventDefault()
  isResizing.value = true
  
  const handleMouseMove = (e: MouseEvent) => {
    if (!contentAreaRef.value) return
    
    const containerRect = contentAreaRef.value.getBoundingClientRect()
    const containerWidth = containerRect.width
    const maxWidth = containerWidth * MAX_KEY_LIST_WIDTH_RATIO
    
    let newWidth = e.clientX - containerRect.left
    newWidth = Math.max(MIN_KEY_LIST_WIDTH, Math.min(newWidth, maxWidth))
    
    keyListWidth.value = Math.round(newWidth)
  }
  
  const handleMouseUp = () => {
    isResizing.value = false
    localStorage.setItem(KEY_LIST_WIDTH_KEY, String(keyListWidth.value))
    document.removeEventListener('mousemove', handleMouseMove)
    document.removeEventListener('mouseup', handleMouseUp)
  }
  
  document.addEventListener('mousemove', handleMouseMove)
  document.addEventListener('mouseup', handleMouseUp)
}

const loadKeyListWidth = () => {
  const savedWidth = localStorage.getItem(KEY_LIST_WIDTH_KEY)
  if (savedWidth) {
    const width = parseInt(savedWidth, 10)
    if (!isNaN(width) && width >= MIN_KEY_LIST_WIDTH) {
      keyListWidth.value = width
    }
  }
}
// ====================================

// 状态
const showSelectServerDialog = ref(false)
const isGlobalLoading = ref(false)
const loadingMessage = ref('正在加载...')
// 保存切换前的连接信息（用于取消）
const previousServer = ref<any>(null)
const previousDb = ref<number | null>(null)
const selectedServer = computed(() => sessionManager.active?.selectedServer ?? null)
const selectedDb = computed(() => sessionManager.active?.selectedDb)
const databases = computed(() => sessionManager.active?.databases || [])
const keys = computed(() => sessionManager.active?.keys || [])

// 排序后的 keys
const sortedKeys = computed(() => {
  if (sessionManager.active.sortOrder === 'asc') {
    return [...keys.value].sort((a, b) => a.localeCompare(b))
  } else if (sessionManager.active.sortOrder === 'desc') {
    return [...keys.value].sort((a, b) => b.localeCompare(a))
  }
  return keys.value
})

// 根据搜索选项过滤后的 keys（用于客户端实时搜索）
const filteredKeys = computed(() => {
  const pattern = searchPattern.value.trim()
  if (!pattern) return sortedKeys.value
  if (caseSensitive.value) {
    return sortedKeys.value.filter(k => k.includes(pattern))
  } else {
    const lower = pattern.toLowerCase()
    return sortedKeys.value.filter(k => k.toLowerCase().includes(lower))
  }
})

// 搜索框 placeholder 动态生成
const searchPlaceholder = computed(() => {
  const scope = searchAll.value ? '搜索全部 key' : '搜索已加载 key'
  const caseInfo = caseSensitive.value ? '（区分大小写）' : ''
  return scope + caseInfo
})

// 处理搜索选项命令
const handleSearchOptionCommand = (command: string) => {
  const session = sessionManager.active
  if (command === 'toggleCaseSensitive') {
    session.caseSensitive = !session.caseSensitive
  } else if (command === 'toggleSearchAll') {
    session.searchAll = !session.searchAll
  }
}

// 处理排序命令
const handleSortCommand = (command: string) => {
  const session = sessionManager.active
  if (command === session.sortOrder) {
    // 再次点击同一排序，取消排序
    session.sortOrder = 'none'
  } else {
    session.sortOrder = command as 'asc' | 'desc'
  }
}

// ========== 搜索选项状态 ==========
const caseSensitive = computed(() => sessionManager.active?.caseSensitive)
const searchAll = computed(() => sessionManager.active?.searchAll)

// ========== 分页加载相关状态 ==========
const keysCursor = computed(() => sessionManager.active?.keysCursor)
const keysTotal = computed(() => sessionManager.active?.keysTotal)
const isLoadingMore = computed(() => sessionManager.active?.isLoadingMore)
const isLoadingAll = computed(() => sessionManager.active?.isLoadingAll)
const loadedCount = computed(() => keys.value.length)
const hasMoreKeys = computed(() => keysCursor.value !== 0 || loadedCount.value < keysTotal.value)
// ====================================

// 消息提示相关
const message = computed(() => sessionManager.active?.message)
const messageType = computed(() => sessionManager.active?.messageType)
const selectedKey = computed(() => sessionManager.active?.selectedKey)
const keyType = computed(() => sessionManager.active?.keyType)
const searchPattern = computed(() => sessionManager.active?.searchPattern)
const selectedDbsForDelete = computed(() => sessionManager.active?.selectedDbsForDelete)
const newDbNumber = computed(() => sessionManager.active?.newDbNumber)
// 导出配置
const exportFolderPath = computed(() => sessionManager.active?.exportFolderPath)
const isFolderLoading = computed(() => sessionManager.active?.isFolderLoading)
// 清空配置
// 多选模式
const isMultiSelectMode = computed(() => sessionManager.active?.isMultiSelectMode)
const selectedKeys = computed(() => sessionManager.active?.selectedKeys)
const treeRef = ref<any>(null)
// 多选面板折叠状态（记住用户偏好）
const MULTI_SELECT_PANEL_KEY = 'redis-helper-multi-select-panel'
const isMultiSelectPanelExpanded = ref<boolean>(
  localStorage.getItem(MULTI_SELECT_PANEL_KEY) !== 'false'
)
// 监听折叠状态变化
watch(isMultiSelectPanelExpanded, (val) => {
  localStorage.setItem(MULTI_SELECT_PANEL_KEY, String(val))
})
// 废键箱视图
const isTrashView = computed(() => sessionManager.active?.isTrashView)
const trashSelectedIds = computed(() => sessionManager.active?.trashSelectedIds)

// 是否显示 key-list-footer（多选面板、加载进度、加载按钮）
const showKeyListFooter = computed(() => {
  if (isTrashView.value) return false
  // 多选模式
  if (isMultiSelectMode.value) return true
  // 正在加载所有
  if (isLoadingAll.value) return true
  // 搜索全部模式且有搜索关键词
  if (searchPattern.value.trim() && searchAll.value) return true
  // 有未加载完的 keys
  if ((hasMoreKeys.value || isLoadingAll.value) && keysTotal.value > 0) return true
  return false
})


// 切换数据库选择状态
const toggleDbSelection = (db: number) => {
  const index = selectedDbsForDelete.value.indexOf(db)
  if (index === -1) {
    sessionManager.active.selectedDbsForDelete.push(db)
  } else {
    sessionManager.active.selectedDbsForDelete.splice(index, 1)
  }
}

// 处理连接下拉菜单命令
const handleDeviceCommand = async (command: any) => {
  // 立即关闭下拉菜单
  if (serverDropdownRef.value?.hide) {
    serverDropdownRef.value.hide()
  }
  
  if (command === 'settings') {
    // 显示服务器配置页面
    sessionManager.active.showServerConfig = true
  } else if (command.type === 'select') {
    // 【第一时间】显示加载遮罩（使用 requestAnimationFrame 确保立即显示）
    isGlobalLoading.value = true
    loadingMessage.value = `正在连接 ${command.server.name}...`
    
    // 保存切换前的连接信息（用于取消）
    previousServer.value = sessionManager.active.selectedServer
    previousDb.value = sessionManager.active.selectedDb
    
    console.log(`[DEBUG] 点击连接 ${command.server.name} (${command.server.host})`)
    const t0 = performance.now()
    
    // 使用 requestAnimationFrame 在下一帧执行耗时操作
    // 这样可以确保加载组件立即显示出来，不被阻塞
    requestAnimationFrame(() => {
      console.log(`[DEBUG] requestAnimationFrame 回调，UI 已更新，耗时 ${performance.now() - t0}ms`)
      const t1 = performance.now()
      // 清空当前数据
      resetSessionData()
      // 设置新服务器
      sessionManager.active.selectedServer = command.server
      // 开始加载数据
      handleServerChange().then(() => {
        console.log(`[DEBUG] handleServerChange 完成，总耗时 ${performance.now() - t1}ms`)
      })
    })
  }
}

// 取消连接切换，返回原来的连接
const cancelConnectionSwitch = () => {
  // 如果有原连接，恢复原来的连接
  if (previousServer.value) {
    // 显示加载状态
    isGlobalLoading.value = true
    loadingMessage.value = `正在恢复 ${previousServer.value.name}...`
    
    // 恢复连接
    sessionManager.active.selectedServer = previousServer.value
    sessionManager.active.selectedDb = previousDb.value
    
    // 清空并重新加载
    requestAnimationFrame(async () => {
      await handleServerChange()
      sessionManager.active.messageType = 'success'
      sessionManager.active.message = '已取消切换连接'
    })
  } else {
    // 没有原连接，直接隐藏遮罩
    isGlobalLoading.value = false
  }
  
  // 清空保存的原连接信息
  previousServer.value = null
  previousDb.value = null
}

// 重置会话数据（切换服务器时调用）
const resetSessionData = () => {
  const session = sessionManager.active
  // 清空数据库列表
  session.databases = []
  // 清空键列表
  session.keys = []
  session.keysCursor = 0
  session.keysTotal = 0
  // 清空选中的键
  session.selectedKey = ''
  session.keyValue = ''
  session.keyType = ''
  // 重置搜索
  session.searchPattern = ''
  // 清空访问记录
  session.visitedDbs.clear()
  session.newlyCreatedDbs.clear()
  // 退出多选模式
  if (session.isMultiSelectMode) {
    session.clearSelection()
  }
  // 退出废键箱视图
  session.isTrashView = false
  // 更新标题
  session.updateTitle()
}

// 从标签栏新建标签页
const handleNewTabFromTabBar = () => {
  // 创建新会话但不切换活动会话，保持当前页面不变
  const newSession = sessionManager.createSession('新标签', undefined, false)
  // 设置新会话为选择服务器状态
  newSession.isSelectingServer = true
  // 显示选择连接对话框
  showSelectServerDialog.value = true
}

// 新建标签页 - 选择服务器
const handleNewTabSelectServer = async (server: any) => {
  // 找到正在选择服务器的会话
  const selectingSession = sessionManager.sessions.find(
    s => s.isSelectingServer
  )
  
  if (selectingSession) {
    selectingSession.isSelectingServer = false
    selectingSession.selectedServer = server
    selectingSession.updateTitle()
    // 切换到新会话
    sessionManager.activeSessionId = selectingSession.id
    await loadDatabases()
    // 关闭选择连接对话框
    showSelectServerDialog.value = false
  }
}

// 新建标签页 - 取消选择，关闭该标签页
const handleCancelNewTab = () => {
  // 关闭选择连接对话框
  showSelectServerDialog.value = false
  
  // 找到正在选择服务器的会话
  const selectingSession = sessionManager.sessions.find(
    s => s.isSelectingServer
  )
  
  if (selectingSession) {
    // 如果有多个会话，关闭新会话
    if (sessionManager.sessions.length > 1) {
      sessionManager.closeSession(selectingSession.id)
    } else {
      // 如果只有一个会话，取消选择状态
      selectingSession.isSelectingServer = false
      if (servers.value.length === 0) {
        selectingSession.showServerConfig = true
        selectingSession.requireServerConnection = true
      }
    }
  }
}

// 从选择连接弹框跳转到服务器配置
const openServerConfigFromDialog = () => {
  sessionManager.active.isSelectingServer = false
  sessionManager.active.showServerConfig = true
}

// 处理DB下拉菜单命令
const handleDbCommand = async (command: any) => {
  if (command === 'add') {
    // 新增DB逻辑
    sessionManager.active.showAddDbDialog = true
  } else if (command === 'delete') {
    // 删除DB逻辑 - 只读模式下禁止
    if (isCurrentServerReadonly.value) {
      sessionManager.active.messageType = 'error'
      sessionManager.active.message = '当前服务器为只读模式，无法删除DB'
      return
    }
    sessionManager.active.showDeleteDbDialog = true
  } else if (command === 'trash') {
    sessionManager.active.isTrashView = true
    await loadTrashItems()
  } else if (command.type === 'select') {
    sessionManager.active.selectedDb = command.db
    sessionManager.active.visitedDbs.add(command.db)
    sessionManager.active.isTrashView = false
    sessionManager.active.updateTitle()
    handleDbChange()
  }
}

// 处理刷新操作
const handleRefresh = async () => {
  // 清空搜索框
  sessionManager.active.searchPattern = ''
  // 重新加载键列表（重置分页状态）
  await loadKeys(true)
}

// 处理更多操作命令
const handleActionCommand = (command: string) => {
  switch (command) {
    case 'add':
      sessionManager.active.showAddKeyDialog = true
      break
    case 'import':
      importData()
      break
    case 'export':
      exportData()
      break
    case 'flush':
      if (isCurrentServerReadonly.value) {
        sessionManager.active.messageType = 'error'
        sessionManager.active.message = '当前服务器为只读模式，无法清空数据库'
        break
      }
      sessionManager.active.showFlushDialog = true
      break
    case 'generateTestData':
      handleGenerateTestData()
      break
    case 'memory':
      handleOpenMemory()
      break
    case 'log':
      handleOpenLog()
      break
  }
}

// 打开内存分析对话框
const handleOpenMemory = async () => {
  const srv = selectedServer.value
  const db = selectedDb.value
  
  if (!srv) {
    sessionManager.active.messageType = 'error'
    sessionManager.active.message = '请先连接到一个 Redis 服务器'
    return
  }
  
  if (db === null) {
    sessionManager.active.messageType = 'error'
    sessionManager.active.message = '请先选择一个数据库'
    return
  }
  
  showMemoryDialog.value = true
  
  // 加载内存信息
  await nextTick()
  if (memoryDialogRef.value) {
    await memoryDialogRef.value.load({
      host: srv.host,
      port: srv.port,
      password: srv.password,
      db: db
    })
  }
}

// 打开日志对话框
const handleOpenLog = () => {
  const srv = selectedServer.value
  if (!srv) {
    sessionManager.active.messageType = 'error'
    sessionManager.active.message = '请先连接到一个 Redis 服务器'
    return
  }
  // 注册当前连接到 logStore（幂等，重复注册无副作用）
  logStore.registerServer(srv.id, srv.name, srv.host, srv.port, srv.password)
  showLogDialog.value = true
  // 自动加载历史
  logStore.setActiveServer(srv.id)
}

// 生成测试数据
const handleGenerateTestData = async () => {
  const server = selectedServer.value
  const db = selectedDb.value
  if (!server || db === null) return

  try {
    await ElMessageBox.confirm(
      `确定要在当前数据库(DB ${db})生成100个测试键吗？`,
      '生成测试数据',
      { confirmButtonText: '确认生成', cancelButtonText: '取消', type: 'info' }
    )

    sessionManager.active.message = ''
    await redis.generateTestData({
      host: server.host,
      port: server.port,
      password: server.password,
      db
    }, 100)

    await loadKeys()
    await loadDatabases()

    sessionManager.active.messageType = 'success'
    sessionManager.active.message = '已生成100个测试键'
  } catch (error: any) {
    if (error === 'cancel' || error?.toString?.().includes('cancel')) return
    sessionManager.active.messageType = 'error'
    sessionManager.active.message = `生成测试数据失败: ${error.message || error}`
  }
}

const handleFlush = async () => {
  const server = selectedServer.value
  const db = selectedDb.value
  if (!server || db === null) return

  try {
    // 安全验证：要求输入服务器名称确认
    const serverName = server.name
    await ElMessageBox.prompt(
      `此操作将清空 DB ${selectedDb.value} 中的所有数据，不可恢复！\n\n请输入当前服务器名称 【${serverName}】 以确认操作：`,
      '清空数据库 - 安全验证',
      {
        confirmButtonText: '确认清空',
        cancelButtonText: '取消',
        type: 'warning',
        inputPlaceholder: `请输入 ${serverName}`,
        inputValidator: (val: string) => {
          if (!val || val.trim().toLowerCase() !== serverName.trim().toLowerCase()) {
            return '服务器名称不匹配，请重新输入'
          }
          return true
        }
      }
    )

    sessionManager.active.message = ''
    const flushedDb = db // 暂存当前DB编号，清空后后端不会返回它

    await redis.flushDatabase({
      host: server.host,
      port: server.port,
      password: server.password,
      db
    })

    sessionManager.active.showFlushDialog = false

    sessionManager.active.clearKeyDetail()

    // 刷新DB列表，同时确保清空的DB仍然显示（计数为0）
    sessionManager.active.newlyCreatedDbs.add(flushedDb)
    await loadDatabases()
    await loadKeys()

    sessionManager.active.messageType = 'success'
    sessionManager.active.message = '数据库清空成功'
  } catch (error: any) {
    if (error === 'cancel' || error?.toString?.().includes('cancel')) return
    console.error('清空失败:', error)
    sessionManager.active.showFlushDialog = false
    sessionManager.active.messageType = 'error'
    sessionManager.active.message = `清空失败: ${error.message || error}`
  }
}

// 各类型示例提示
const typePlaceholders: Record<string, string> = {
  string: '输入字符串值',
  list: '输入 JSON 数组格式',
  set: '输入 JSON 数组格式',
  zset: '输入 JSON 数组格式',
  hash: '输入 JSON 对象格式\n示例：\n{\n  "field1": "value1",\n  "field2": "value2"\n}'
}

const typeExamples: Record<string, string> = {
  string: 'Hello world\n或者\n{"name":"张三","age":25}',
  list: '["item1", "item2", "item3"]',
  set: '["member1", "member2", "member3"]',
  zset: '[["player1", 100], ["player2", 80]]',
  hash: '{\n  "field1": "value1",\n  "field2": "value2"\n}'
}

// ========== 元素编辑模式相关 ==========
const COMPLEX_TYPES = ['list', 'set', 'zset', 'hash']

// 元素数据结构
interface ElementItem {
  _index: number
  value: string
  field?: string
  score?: number
}

// 是否为复杂类型
const isComplexType = computed(() => COMPLEX_TYPES.includes(keyType.value))

// 元素搜索关键词
const elementSearchKeyword = ref('')

// ZSet 排序方式
const zsetSortOrder = ref<'default' | 'score-asc' | 'score-desc' | 'member-asc' | 'member-desc'>('default')

// 当前正在编辑的行索引（-1 表示未在编辑）
const editingIndex = ref(-1)

// 编辑缓冲区
const editBuffer = ref<{ field?: string; value: string; score?: number; _index?: number }>({ value: '', _index: -1 })

// 解析元素：将 JSON 字符串解析为结构化数组
const parsedElements = computed<ElementItem[]>(() => {
  if (!isComplexType.value || !editKeyForm.value?.value) return []
  try {
    const parsed = JSON.parse(editKeyForm.value.value)
    const type = keyType.value
    if (type === 'list' || type === 'set') {
      return Array.isArray(parsed) ? parsed.map((v, i) => ({ _index: i, value: String(v) })) : []
    }
    if (type === 'zset') {
      return Array.isArray(parsed)
        ? parsed.map((item: any[], i) => ({
            _index: i,
            value: String(item[0] ?? ''),
            score: Number(item[1] ?? 0)
          }))
        : []
    }
    if (type === 'hash' && typeof parsed === 'object' && !Array.isArray(parsed)) {
      return Object.entries(parsed).map(([k, v], i) => ({ _index: i, field: k, value: String(v) }))
    }
    return []
  } catch {
    // 解析失败时返回空数组，提示用户使用覆盖模式
    return []
  }
})

// 搜索过滤后的元素（ZSet 应用排序）
const filteredElements = computed(() => {
  let result = parsedElements.value

  // 关键字过滤
  if (elementSearchKeyword.value.trim()) {
    const keyword = elementSearchKeyword.value.toLowerCase()
    result = result.filter(el => {
      if (keyType.value === 'hash') {
        return (el.field?.toLowerCase().includes(keyword) ?? false) ||
               el.value.toLowerCase().includes(keyword)
      }
      if (keyType.value === 'zset') {
        return el.value.toLowerCase().includes(keyword) ||
               String(el.score ?? '').includes(keyword)
      }
      return el.value.toLowerCase().includes(keyword)
    })
  }

  // ZSet 排序（基于副本，不影响原始 _index）
  if (keyType.value === 'zset' && zsetSortOrder.value !== 'default') {
    result = [...result]
    switch (zsetSortOrder.value) {
      case 'score-asc':
        result.sort((a, b) => (a.score ?? 0) - (b.score ?? 0))
        break
      case 'score-desc':
        result.sort((a, b) => (b.score ?? 0) - (a.score ?? 0))
        break
      case 'member-asc':
        result.sort((a, b) => a.value.localeCompare(b.value))
        break
      case 'member-desc':
        result.sort((a, b) => b.value.localeCompare(a.value))
        break
    }
  }

  return result
})

// 格式化元素显示
const formatElementDisplay = (row: any): string => {
  const type = keyType.value
  if (type === 'zset') {
    return `${row.value} → ${row.score}`
  }
  if (type === 'hash') {
    return `${row.field}: ${row.value}`
  }
  return row.value
}

// 查看元素详情（大文本预览）
const viewElement = async (row: any) => {
  const content = formatElementDisplay(row)
  await ElMessageBox.alert(content, '元素详情', { confirmButtonText: '关闭' })
}

// 开始编辑某一行
const startEdit = (index: number) => {
  const realIndex = filteredElements.value[index]._index
  const original = parsedElements.value[realIndex]
  if (!original) return
  editingIndex.value = index
  editBuffer.value = {
    ...(original.field !== undefined ? { field: original.field } : {}),
    value: original.value,
    ...(original.score !== undefined ? { score: original.score } : {})
  }
}

// 取消编辑
const cancelEdit = () => {
  editingIndex.value = -1
  editBuffer.value = { value: '', _index: -1 }
}

// 添加新元素 - 打开对话框
const showAddElementDialog = ref(false)
const isAddingElement = ref(false)
const addElementForm = ref<{ field?: string; value: string; score: string }>({ value: '', score: '0' })

// 添加元素对话框标题
const addElementDialogTitle = computed(() => {
  const typeMap: Record<string, string> = {
    list: '添加 List 元素',
    set: '添加 Set 成员',
    zset: '添加 ZSet 成员',
    hash: '添加 Hash 字段'
  }
  return typeMap[keyType.value] || '添加元素'
})

// 打开添加元素对话框
const openAddElementDialog = () => {
  cancelEdit()
  // 根据类型初始化表单
  addElementForm.value = {
    value: '',
    score: '0',
    ...(keyType.value === 'hash' ? { field: '' } : {})
  }
  showAddElementDialog.value = true
}

// 确认添加元素
const confirmAddElement = async () => {
  if (!selectedServer.value || !editKeyForm.value?.key) return

  const type = keyType.value
  const form = addElementForm.value
  const key = editKeyForm.value.key

  // 校验必填字段
  if (!form.value?.trim()) {
    ElMessage.warning('值不能为空')
    return
  }
  if (type === 'hash' && !form.field?.trim()) {
    ElMessage.warning('Field 不能为空')
    return
  }
  if (type === 'zset') {
    const scoreNum = Number(form.score)
    if (form.score === '' || isNaN(scoreNum)) {
      ElMessage.warning('Score 必须是数字')
      return
    }
  }

  // Hash 重复字段确认
  if (type === 'hash' && form.field) {
    const exists = parsedElements.value.some(el => el.field === form.field)
    if (exists) {
      try {
        await ElMessageBox.confirm(
          `Field "${form.field}" 已存在，添加后将覆盖原值，是否继续？`,
          'Field 已存在',
          { confirmButtonText: '覆盖', cancelButtonText: '取消', type: 'warning' }
        )
      } catch {
        return
      }
    }
  }

  isAddingElement.value = true
  try {
    sessionManager.active.message = ''

    // 构造 buffer 并调用 Redis 添加命令
    const buffer: any = { value: form.value }
    if (type === 'hash') buffer.field = form.field
    if (type === 'zset') buffer.score = Number(form.score)

    await addElementToRedis(type, key, buffer)

    // 刷新数据显示
    await loadKeyValue(key)
    showAddElementDialog.value = false
    sessionManager.active.messageType = 'success'
    sessionManager.active.message = '元素添加成功'
  } catch (error: any) {
    console.error('添加元素失败:', error)
    sessionManager.active.messageType = 'error'
    sessionManager.active.message = `添加失败: ${error.message || error}`
  } finally {
    isAddingElement.value = false
  }
}

// 保存元素（仅修改现有元素，新增走对话框）
const saveElement = async (tableIndex: number) => {
  if (!selectedServer.value || !editKeyForm.value?.key) return

  const type = keyType.value
  const buffer = editBuffer.value
  const key = editKeyForm.value.key

  // 校验必填字段
  if (!buffer.value?.trim()) {
    ElMessage.warning('值不能为空')
    return
  }
  if (type === 'hash' && !buffer.field?.trim()) {
    ElMessage.warning('Field 不能为空')
    return
  }
  if (type === 'zset' && (buffer.score === undefined || isNaN(buffer.score))) {
    ElMessage.warning('Score 必须是数字')
    return
  }

  try {
    sessionManager.active.message = ''

    // 修改现有元素
    const realIndex = filteredElements.value[tableIndex]?._index
    if (realIndex === undefined || realIndex < 0) {
      ElMessage.warning('无法定位到要修改的元素')
      return
    }
    await updateElementInRedis(type, key, realIndex, buffer, parsedElements.value)

    // 刷新数据显示
    await loadKeyValue(key)
    cancelEdit()
    sessionManager.active.messageType = 'success'
    sessionManager.active.message = '元素修改成功'
  } catch (error: any) {
    console.error('保存元素失败:', error)
    sessionManager.active.messageType = 'error'
    sessionManager.active.message = `操作失败: ${error.message || error}`
  }
}

// 调用 Redis 命令添加元素
const addElementToRedis = async (type: string, key: string, buffer: any) => {
  const conn = selectedServer.value!
  if (type === 'list') {
    await redis.rpush({ host: conn.host, port: conn.port, password: conn.password, db: selectedDb.value ?? 0, key, value: buffer.value })
  } else if (type === 'set') {
    await redis.sadd({ host: conn.host, port: conn.port, password: conn.password, db: selectedDb.value ?? 0, key, values: [buffer.value] })
  } else if (type === 'zset') {
    await redis.zadd({ host: conn.host, port: conn.port, password: conn.password, db: selectedDb.value ?? 0, key, members: [[buffer.value, buffer.score]] })
  } else if (type === 'hash') {
    await redis.hset({ host: conn.host, port: conn.port, password: conn.password, db: selectedDb.value ?? 0, key, field: buffer.field, value: buffer.value })
  }
}

// 调用 Redis 命令更新现有元素
const updateElementInRedis = async (type: string, key: string, index: number, buffer: any, allElements: any[]) => {
  const conn = selectedServer.value!
  const original = allElements[index]
  if (!original) return

  if (type === 'list') {
    // List 通过 LSET 更新指定索引的值
    await redis.lset({ host: conn.host, port: conn.port, password: conn.password, db: selectedDb.value ?? 0, key, index, value: buffer.value })
  } else if (type === 'set') {
    // Set 是无序集合：先删除旧值，再添加新值
    if (original.value !== buffer.value) {
      await redis.srem({ host: conn.host, port: conn.port, password: conn.password, db: selectedDb.value ?? 0, key, values: [original.value] })
      await redis.sadd({ host: conn.host, port: conn.port, password: conn.password, db: selectedDb.value ?? 0, key, values: [buffer.value] })
    }
  } else if (type === 'zset') {
    // ZSet：先删除旧成员，再添加新成员（如果 member 变化）或仅更新分数
    if (original.value !== buffer.value) {
      await redis.zrem({ host: conn.host, port: conn.port, password: conn.password, db: selectedDb.value ?? 0, key, members: [original.value] })
      await redis.zadd({ host: conn.host, port: conn.port, password: conn.password, db: selectedDb.value ?? 0, key, members: [[buffer.value, buffer.score]] })
    } else if (original.score !== buffer.score) {
      // 仅分数变化，使用 ZADD 覆盖
      await redis.zadd({ host: conn.host, port: conn.port, password: conn.password, db: selectedDb.value ?? 0, key, members: [[buffer.value, buffer.score]] })
    }
  } else if (type === 'hash') {
    // Hash 直接 HSET 更新或新增 field
    await redis.hset({ host: conn.host, port: conn.port, password: conn.password, db: selectedDb.value ?? 0, key, field: buffer.field || original.field, value: buffer.value })
    // 如果 field 名变了，需要删除旧的
    if (buffer.field && original.field && buffer.field !== original.field) {
      await redis.hdel({ host: conn.host, port: conn.port, password: conn.password, db: selectedDb.value ?? 0, key, fields: [original.field] })
    }
  }
}

// 删除元素
const removeElement = async (tableIndex: number) => {
  if (!selectedServer.value || !editKeyForm.value?.key) return

  const type = keyType.value
  const key = editKeyForm.value.key
  const realIndex = filteredElements.value[tableIndex]?._index
  const original = realIndex !== undefined ? parsedElements.value[realIndex] : null
  if (!original) return

  // 根据类型构造确认消息
  let confirmMsg = ''
  if (type === 'hash') {
    confirmMsg = `确定要删除 Field "${original.field}" 吗？`
  } else if (type === 'zset') {
    confirmMsg = `确定要删除成员 "${original.value}" (Score: ${original.score}) 吗？`
  } else {
    confirmMsg = `确定要删除 "${original.value}" 吗？`
  }

  try {
    await ElMessageBox.confirm(confirmMsg, '确认删除', {
      confirmButtonText: '确认删除',
      cancelButtonText: '取消',
      type: 'warning'
    })

    sessionManager.active.message = ''
    const conn = selectedServer.value!

    // 调用对应的 Redis 删除命令
    if (type === 'list') {
      // List 需要通过值来删除（可能有多处相同值）
      await redis.lrem({ host: conn.host, port: conn.port, password: conn.password, db: selectedDb.value ?? 0, key, count: 1, value: original.value })
    } else if (type === 'set') {
      await redis.srem({ host: conn.host, port: conn.port, password: conn.password, db: selectedDb.value ?? 0, key, values: [original.value] })
    } else if (type === 'zset') {
      await redis.zrem({ host: conn.host, port: conn.port, password: conn.password, db: selectedDb.value ?? 0, key, members: [original.value] })
    } else if (type === 'hash') {
      await redis.hdel({ host: conn.host, port: conn.port, password: conn.password, db: selectedDb.value ?? 0, key, fields: [original.field || ''] })
    }

    // 刷新显示
    await loadKeyValue(key)
    sessionManager.active.messageType = 'success'
    sessionManager.active.message = '元素删除成功'
  } catch (error: any) {
    if (error === 'cancel' || error?.toString?.().includes('cancel')) return
    sessionManager.active.messageType = 'error'
    sessionManager.active.message = `删除失败: ${error.message || error}`
  }
}

// 表单数据
const newKeyForm = computed(() => sessionManager.active?.newKeyForm)

const editKeyForm = computed(() => sessionManager.active?.editKeyForm)

// 计算属性
const servers = computed(() => server.servers)
const isCurrentServerReadonly = computed(() => sessionManager.active?.selectedServer?.readonly === true)
const keyTree = computed(() => {
  return filteredKeys.value.map(key => ({
    id: key,
    label: key
  }))
})



// 方法
const handleServerChange = async () => {
  const session = sessionManager.active
  if (!session.selectedServer) {
    // 如果没有选中服务器，隐藏加载遮罩
    isGlobalLoading.value = false
    return
  }
  
  // 切换服务器时退出多选模式
  if (session.isMultiSelectMode) {
    session.clearSelection()
  }
  
  // 重置 selectedDb 为 null
  session.selectedDb = null
  // 立即更新标题（显示新服务器名称）
  session.updateTitle()
  
  // 直接加载数据库列表（无需 nextTick，因为已通过 requestAnimationFrame 确保 UI 更新）
  await loadDatabases()
  
  // 数据加载完成，隐藏加载遮罩
  isGlobalLoading.value = false
}

const handleDbChange = async () => {
  const session = sessionManager.active
  // 切换 DB 时退出多选模式
  if (session.isMultiSelectMode) {
    session.clearSelection()
  }
  // 切换 DB 时重置分页状态
  await loadKeys(true)
}

const loadDatabases = async () => {
  if (!selectedServer.value) return
  
  try {
    sessionManager.active.message = ''
    const backendDatabases = await redis.getDatabases({
      host: selectedServer.value.host,
      port: selectedServer.value.port,
      password: selectedServer.value.password,
      db: selectedServer.value.db
    })
    
    // 合并后端返回的数据库（有key的）和前端跟踪的数据库
    const backendDbs = new Set(backendDatabases.map(db => db[0]))
    const mergedDbs = [...backendDatabases]

    // 添加前端跟踪的新增数据库（如果后端还没有返回）
    sessionManager.active.newlyCreatedDbs.forEach(dbNum => {
      if (!backendDbs.has(dbNum)) {
        mergedDbs.push([dbNum, 0])
      } else {
        sessionManager.active.newlyCreatedDbs.delete(dbNum)
      }
    })

    // 添加曾经访问过但已变空的数据库
    sessionManager.active.visitedDbs.forEach(dbNum => {
      if (!backendDbs.has(dbNum) && !sessionManager.active.newlyCreatedDbs.has(dbNum)) {
        mergedDbs.push([dbNum, 0])
      }
    })
    
    // 按数据库编号排序
    mergedDbs.sort((a, b) => a[0] - b[0])
    
    sessionManager.active.databases = mergedDbs
    
    // 连接成功后自动选择第一个数据库，加载前100个键（分页显示）
    if (mergedDbs.length > 0 && sessionManager.active.selectedDb == null) {
      const firstDb = mergedDbs[0][0]
      sessionManager.active.selectedDb = firstDb
      sessionManager.active.visitedDbs.add(firstDb)
      // 加载前100个键（默认分页大小）
      await loadKeys(true)
    }
    
    // 更新标签标题（显示服务器名称和当前DB）
    sessionManager.active.updateTitle()
  } catch (error: any) {
    console.error('加载数据库失败:', error)
    sessionManager.active.messageType = 'error'
    sessionManager.active.message = `加载数据库失败: ${error.message || error}`
  }
}

const loadKeys = async (reset: boolean = true) => {
  if (!selectedServer.value) return

  try {
    sessionManager.active.message = ''

    if (reset) {
      // 重置状态，首次加载
      sessionManager.active.keys = []
      sessionManager.active.keysCursor = 0
    }

    // 分页加载 - 每次加载100个
    const pageSize = 100

    // 首次加载时限制为100个，后续加载获取全部
    const keysResponse = await redis.getKeys({
      host: selectedServer.value.host,
      port: selectedServer.value.port,
      password: selectedServer.value.password,
      db: selectedDb.value ?? 0,
      ...(reset ? { limit: pageSize } : {}) // 首次加载限制100个
    })

    const allKeys = keysResponse.keys
    const dbTotal = keysResponse.total

    // 检查 allKeys 是否为数组
    if (!Array.isArray(allKeys)) {
      console.error('加载键失败: keys 不是数组', allKeys)
      sessionManager.active.messageType = 'error'
      sessionManager.active.message = '加载键失败: 数据格式错误'
      return
    }

    // 设置总数（使用后端返回的真实总数）
    sessionManager.active.keysTotal = dbTotal

    // 如果是首次加载，只取前100个
    if (reset) {
      sessionManager.active.keys = allKeys.slice(0, pageSize)
      if (allKeys.length >= pageSize) {
        sessionManager.active.keysCursor = pageSize // 标记还有更多
      } else {
        sessionManager.active.keysCursor = 0 // 没有更多了
      }
    } else {
      // 追加加载
      sessionManager.active.keys = allKeys
      sessionManager.active.keysCursor = 0
    }

    sessionManager.active.selectedKey = ''
    sessionManager.active.keyValue = ''
    sessionManager.active.keyType = ''
  } catch (error: any) {
    console.error('加载键失败:', error)
    sessionManager.active.messageType = 'error'
    sessionManager.active.message = `加载键失败: ${error.message || error}`
  }
}

// 加载更多 - 每次加载100个
const handleLoadMore = async () => {
  if (!selectedServer.value || isLoadingMore.value) return

  sessionManager.active.isLoadingMore = true
  const startTime = Date.now()
  try {
    const pageSize = 100

    // 获取所有 keys（实际应该用分页 API）
    const keysResponse = await redis.getKeys({
      host: selectedServer.value.host,
      port: selectedServer.value.port,
      password: selectedServer.value.password,
      db: selectedDb.value ?? 0
    })

    const allKeys = keysResponse.keys
    const dbTotal = keysResponse.total

    // 检查 allKeys 是否为数组
    if (!Array.isArray(allKeys)) {
      console.error('加载键失败: keys 不是数组', allKeys)
      sessionManager.active.messageType = 'error'
      sessionManager.active.message = '加载键失败: 数据格式错误'
      return
    }

    // 获取当前已加载的数量
    const currentLength = keys.value.length
    // 计算剩余数量
    const remaining = allKeys.length - currentLength

    // 更新总数
    sessionManager.active.keysTotal = dbTotal

    if (remaining > 0) {
      // 加载下一批（最多 pageSize 个）
      const nextBatch = allKeys.slice(currentLength, currentLength + pageSize)
      sessionManager.active.keys = [...keys.value, ...nextBatch]

      // 更新游标
      if (currentLength + pageSize < allKeys.length) {
        sessionManager.active.keysCursor = currentLength + pageSize
      } else {
        sessionManager.active.keysCursor = 0 // 没有更多了
      }

      // 等待 DOM 更新后滚动到新内容
      await nextTick()
      const treeEl = document.querySelector('.key-list-content .el-scrollbar__wrap')
      if (treeEl) {
        const scrollTarget = treeEl.scrollHeight - treeEl.clientHeight - 50
        treeEl.scrollTo({ top: Math.max(0, scrollTarget), behavior: 'smooth' })
      }

      // 数量文字短暂高亮
      const countEl = document.querySelector('.count-divider')
      if (countEl) {
        countEl.classList.add('count-highlight')
        setTimeout(() => countEl.classList.remove('count-highlight'), 600)
      }
    }

    // 确保 loading 状态至少显示 300ms
    const elapsed = Date.now() - startTime
    if (elapsed < 300) {
      await new Promise(resolve => setTimeout(resolve, 300 - elapsed))
    }
  } catch (error: any) {
    console.error('加载更多失败:', error)
    sessionManager.active.messageType = 'error'
    sessionManager.active.message = `加载更多失败: ${error.message || error}`
  } finally {
    sessionManager.active.isLoadingMore = false
  }
}

// 加载所有 keys，带真实的分批进度显示
const handleLoadAll = async () => {
  if (!selectedServer.value || isLoadingAll.value) return

  sessionManager.active.isLoadingAll = true
  try {
    // 先获取所有 keys 用于计算总数
    const keysResponse = await redis.getKeys({
      host: selectedServer.value.host,
      port: selectedServer.value.port,
      password: selectedServer.value.password,
      db: selectedDb.value ?? 0
    })

    const allKeys = keysResponse.keys
    const total = keysResponse.total
    sessionManager.active.keysTotal = total
    const batchSize = 100 // 每批加载 100 个

    // 检查 allKeys 是否为数组
    if (!Array.isArray(allKeys)) {
      console.error('加载键失败: keys 不是数组', allKeys)
      sessionManager.active.messageType = 'error'
      sessionManager.active.message = '加载键失败: 数据格式错误'
      return
    }

    // 如果总数较少，直接显示
    if (total <= batchSize) {
      sessionManager.active.keys = allKeys
      sessionManager.active.keysCursor = 0
      sessionManager.active.messageType = 'success'
      sessionManager.active.message = `已加载全部 ${total} 个 keys`
      return
    }

    // 分批逐步加载，营造真实进度感
    sessionManager.active.keys = []
    for (let i = 0; i < total; i += batchSize) {
      const batch = allKeys.slice(i, i + batchSize)
      sessionManager.active.keys = [...keys.value, ...batch]

      // 等待 Vue 更新 DOM，确保进度条渲染
      await nextTick()
      // 额外等待，让 CSS transition 有时间播放
      await new Promise(resolve => setTimeout(resolve, 80))
    }

    sessionManager.active.keysCursor = 0 // 没有更多了

    sessionManager.active.messageType = 'success'
    sessionManager.active.message = `已加载全部 ${total} 个 keys`
  } catch (error: any) {
    console.error('加载所有失败:', error)
    sessionManager.active.messageType = 'error'
    sessionManager.active.message = `加载所有失败: ${error.message || error}`
  } finally {
    sessionManager.active.isLoadingAll = false
  }
}

const handleKeyClick = async (node: any) => {
  sessionManager.active.selectedKey = node.id
  await loadKeyValue(node.id)
}

const loadKeyValue = async (key: string) => {
  if (!selectedServer.value) return
  
  try {
    sessionManager.active.message = ''
    const result = await redis.getKeyValue({
      host: selectedServer.value.host,
      port: selectedServer.value.port,
      password: selectedServer.value.password,
      db: selectedDb.value ?? 0,
      key
    })
    sessionManager.active.keyValue = result.value
    sessionManager.active.keyType = result.key_type
    // 格式化显示JSON，其他类型保持原样
    sessionManager.active.editKeyForm = {
      key: key,
      value: formatJsonDisplay(result.value),
      type: result.key_type
    }
  } catch (error: any) {
    console.error('加载键值失败:', error)
    sessionManager.active.messageType = 'error'
    sessionManager.active.message = `加载键值失败: ${error.message || error}`
  }
}

// 刷新当前键值
const refreshKeyValue = async () => {
  if (!selectedKey.value) {
    sessionManager.active.messageType = 'warning'
    sessionManager.active.message = '请先选择一个键'
    return
  }
  await loadKeyValue(selectedKey.value)
  sessionManager.active.messageType = 'success'
  sessionManager.active.message = '刷新成功'
}

// 格式化JSON显示
const formatJsonDisplay = (value: string): string => {
  try {
    return JSON.stringify(JSON.parse(value), null, 2)
  } catch {
    return value
  }
}

// 压缩JSON存储
const compressJson = (value: string): string => {
  try {
    return JSON.stringify(JSON.parse(value))
  } catch {
    return value
  }
}

const searchKeys = async () => {
  if (!selectedServer.value) return

  const input = searchPattern.value.trim()

  // 如果输入不含 Redis 通配符（* ? [），直接走客户端过滤（filteredKeys computed 自动响应）
  // 如果含通配符，走后端精确 pattern 查询
  const hasWildcard = /[*?\[]/.test(input)
  if (!hasWildcard) {
    // 客户端过滤 —— filteredKeys 已自动响应 searchPattern 变化，无需手动操作
    // 如果需要搜索全部且未全部加载，先加载全部
    if (sessionManager.active.searchAll && keysCursor.value !== 0 && input) {
      await handleLoadAll()
    }
    return
  }

  // 含通配符 → 走后端
  try {
    sessionManager.active.message = ''
    const result = await redis.searchKeys({
      host: selectedServer.value.host,
      port: selectedServer.value.port,
      password: selectedServer.value.password,
      db: selectedDb.value ?? 0,
      pattern: input || '*'
    })

    // 后端结果再做一次客户端不区分大小写过滤（兜底）
    const lower = input.replace(/[*?\[]/g, '').toLowerCase()
    sessionManager.active.keys = lower
      ? result.filter(k => k.toLowerCase().includes(lower))
      : result

    sessionManager.active.keysCursor = 0
    sessionManager.active.keysTotal = keys.value.length
  } catch (error: any) {
    console.error('搜索键失败:', error)
    sessionManager.active.messageType = 'error'
    sessionManager.active.message = `搜索键失败: ${error.message || error}`
  }
}

const addKey = async () => {
  if (!selectedServer.value) return

  const type = newKeyForm.value.type
  const key = newKeyForm.value.key.trim()
  const value = newKeyForm.value.value.trim()

  // 校验：请输入键
  if (!key) {
    ElMessageBox.alert('请输入键名称', '提示', { type: 'warning' })
    return
  }

  // 校验：键格式（不允许包含空格和特殊控制字符）
  if (/[\s\x00-\x1f\x7f]/.test(key)) {
    ElMessageBox.alert('键名不能包含空格或特殊字符\n\n示例：user:1001、order_info、my_key', '提示', { type: 'warning' })
    return
  }

  // 校验：请输入值
  if (!value) {
    const typeValueExamples: Record<string, string> = {
      string: 'Hello world',
      list: '["item1", "item2", "item3"]',
      set: '["member1", "member2", "member3"]',
      zset: '[["player1", 100], ["player2", 80]]',
      hash: '{"field1": "value1", "field2": "value2"}'
    }
    ElMessageBox.alert(`请输入键的值\n\n${type} 类型值示例：\n${typeValueExamples[type] || ''}`, '提示', { type: 'warning' })
    return
  }

  // 校验：非 string 类型的值必须是有效 JSON
  if (type !== 'string') {
    try {
      const parsed = JSON.parse(value)
      // 校验 JSON 结构
      if (type === 'list' && !Array.isArray(parsed)) {
        ElMessageBox.alert('列表类型的值必须是 JSON 数组\n\n示例：\n["item1", "item2", "item3"]', '提示', { type: 'warning' })
        return
      }
      if (type === 'set' && !Array.isArray(parsed)) {
        ElMessageBox.alert('集合类型的值必须是 JSON 数组\n\n示例：\n["member1", "member2", "member3"]', '提示', { type: 'warning' })
        return
      }
      if (type === 'zset') {
        if (!Array.isArray(parsed) || !parsed.every(item => Array.isArray(item) && item.length === 2)) {
          ElMessageBox.alert('有序集合类型的值必须是 JSON 数组，每个元素为 [成员, 分数]\n\n示例：\n[["player1", 100], ["player2", 80]]', '提示', { type: 'warning' })
          return
        }
      }
      if (type === 'hash' && (typeof parsed !== 'object' || Array.isArray(parsed) || parsed === null)) {
        ElMessageBox.alert('哈希类型的值必须是 JSON 对象\n\n示例：\n{"field1": "value1", "field2": "value2"}', '提示', { type: 'warning' })
        return
      }
    } catch {
      const typeValueExamples: Record<string, string> = {
        list: '["item1", "item2", "item3"]',
        set: '["member1", "member2", "member3"]',
        zset: '[["player1", 100], ["player2", 80]]',
        hash: '{"field1": "value1", "field2": "value2"}'
      }
      ElMessageBox.alert(`值格式不正确，${type} 类型需要有效的 JSON 格式\n\n示例：\n${typeValueExamples[type] || ''}`, '提示', { type: 'warning' })
      return
    }
  }

  try {
    sessionManager.active.message = ''
    // 压缩JSON后保存
    const compressedValue = compressJson(newKeyForm.value.value)
    await redis.setKeyValue({
      host: selectedServer.value.host,
      port: selectedServer.value.port,
      password: selectedServer.value.password,
      db: selectedDb.value ?? 0,
      key: newKeyForm.value.key,
      value: compressedValue,
      key_type: newKeyForm.value.type
    })
    await loadKeys()
    await loadDatabases()
    sessionManager.active.showAddKeyDialog = false
    // 重置表单
    sessionManager.active.newKeyForm = {
      key: '',
      value: '',
      type: 'string'
    }
  } catch (error: any) {
    console.error('添加键失败:', error)
    ElMessageBox.alert(`添加键失败，请检查输入是否正确`, '错误', { type: 'error' })
  }
}

const updateKey = async () => {
  if (!selectedServer.value || !editKeyForm.value.key) return
  
  try {
    sessionManager.active.message = ''
    // 压缩JSON后保存
    const compressedValue = compressJson(editKeyForm.value.value)
    await redis.setKeyValue({
      host: selectedServer.value.host,
      port: selectedServer.value.port,
      password: selectedServer.value.password,
      db: selectedDb.value ?? 0,
      key: editKeyForm.value.key,
      value: compressedValue,
      key_type: editKeyForm.value.type
    })
    await loadKeyValue(editKeyForm.value.key)
    sessionManager.active.showEditKeyDialog = false
    sessionManager.active.messageType = 'success'
    sessionManager.active.message = '修改成功'
  } catch (error: any) {
    console.error('修改键失败:', error)
    sessionManager.active.messageType = 'error'
    sessionManager.active.message = `修改键失败: ${error.message || error}`
  }
}

const deleteKey = async () => {
  if (!selectedServer.value || !selectedKey.value) return
  if (isCurrentServerReadonly.value) {
    sessionManager.active.messageType = 'error'
    sessionManager.active.message = '当前服务器为只读模式，无法删除Key'
    return
  }
  try {
    await ElMessageBox.confirm('确定要删除该键吗？删除后将删除废键箱，7天后自动清除。', '确认删除', {
      confirmButtonText: '确认删除', cancelButtonText: '取消', type: 'warning',
    })
    sessionManager.active.message = ''
    await trash.moveToTrash({
      host: selectedServer.value.host, port: selectedServer.value.port,
      password: selectedServer.value.password, db: selectedDb.value ?? 0, key: selectedKey.value,
    })
    sessionManager.active.clearKeyDetail()
    await loadKeys()
    await loadDatabases()
    await loadTrashItems()
    sessionManager.active.messageType = 'success'
    sessionManager.active.message = '已删除废键箱，7天后自动清除'
  } catch (error: any) {
    if (error === 'cancel' || error?.toString?.().includes('cancel')) return
    sessionManager.active.messageType = 'error'
    sessionManager.active.message = `删除键失败: ${error.message || error}`
  }
}

const selectExportFolder = async () => {
  try {
    sessionManager.active.isFolderLoading = true
    
    if (isTauriEnv()) {
      // Tauri 环境：使用原生对话框
      const { open } = await import('@tauri-apps/plugin-dialog')
      const selected = await open({
        title: '选择保存文件夹',
        directory: true,
        multiple: false
      })
      
      if (selected !== null && selected !== undefined) {
        let folderPath = ''
        const selectedItems = selected as string | string[]
        if (typeof selectedItems === 'string') {
          folderPath = selectedItems
        } else if (Array.isArray(selectedItems) && selectedItems.length > 0) {
          folderPath = selectedItems[0]
        }
        if (folderPath) {
          sessionManager.active.exportFolderPath = folderPath
        }
      }
    } else {
      // 浏览器环境：导出时直接下载，无需选择文件夹
      sessionManager.active.exportFolderPath = 'browser-download'
    }
  } catch (e) {
    console.error('文件夹选择错误:', e)
    sessionManager.active.messageType = 'error'
    sessionManager.active.message = `文件夹选择失败: ${e}`
  } finally {
    sessionManager.active.isFolderLoading = false
  }
}

const exportData = async () => {
  if (!selectedServer.value) return
  
  console.log('打开导出对话框...')
  
  // 打开导出对话框
  sessionManager.active.exportFileName = 'redis-export'
  // 不重置exportFolderPath，保持用户之前的选择
  sessionManager.active.showExportDialog = true
}

const handleExport = async () => {
  if (!selectedServer.value) return
  
  if (!exportFolderPath.value) {
    sessionManager.active.messageType = 'error'
    sessionManager.active.message = '请先选择保存文件夹'
    return
  }
  
  try {
    sessionManager.active.message = ''
    
    const fileName = sessionManager.active.exportFileName || 'redis-export'
    
    if (isTauriEnv()) {
      // Tauri 环境：通过后端写入文件
      const { resolve } = await import('@tauri-apps/api/path')
      const filePath = await resolve(exportFolderPath.value, `${fileName}.json`)
      
      await redis.exportData({
        host: selectedServer.value.host,
        port: selectedServer.value.port,
        password: selectedServer.value.password,
        db: selectedDb.value ?? 0,
        file_path: filePath
      })
      
      sessionManager.active.showExportDialog = false
      sessionManager.active.messageType = 'success'
      sessionManager.active.message = `导出成功: ${filePath}`
    } else {
      // 浏览器环境：获取数据后下载为文件
      const keysResponse = await redis.getKeys({
        host: selectedServer.value.host,
        port: selectedServer.value.port,
        password: selectedServer.value.password,
        db: selectedDb.value ?? 0
      })
      
      const data = []
      for (const key of keysResponse.keys) {
        try {
          const result = await redis.getKeyValue({
            host: selectedServer.value.host,
            port: selectedServer.value.port,
            password: selectedServer.value.password,
            db: selectedDb.value ?? 0,
            key
          })
          data.push({ key: result.key, value: result.value, type: result.key_type })
        } catch { /* skip */ }
      }
      
      const jsonStr = JSON.stringify(data, null, 2)
      const blob = new Blob([jsonStr], { type: 'application/json' })
      const url = URL.createObjectURL(blob)
      const a = document.createElement('a')
      a.href = url
      a.download = `${fileName}.json`
      a.click()
      URL.revokeObjectURL(url)
      
      sessionManager.active.showExportDialog = false
      sessionManager.active.messageType = 'success'
      sessionManager.active.message = `导出成功: ${fileName}.json`
    }
  } catch (error: any) {
    console.error('导出失败:', error)
    sessionManager.active.messageType = 'error'
    sessionManager.active.message = `导出失败: ${error.message || error}`
  }
}

// 创建隐藏的文件输入元素
const fileInput = ref<HTMLInputElement | null>(null)
const folderInput = ref<HTMLInputElement | null>(null)

// 生命周期
onMounted(async () => {
  try {
    sessionManager.init()
    sessionManager.active.message = ''
    
    // 加载保存的分隔条宽度
    loadKeyListWidth()
    
    // 导入文件选择
    fileInput.value = document.createElement('input')
    fileInput.value.type = 'file'
    fileInput.value.accept = '.json'
    fileInput.value.style.display = 'none'
    document.body.appendChild(fileInput.value)
    
    // 监听文件选择事件
    fileInput.value.addEventListener('change', async (event) => {
      const target = event.target as HTMLInputElement
      if (target.files && target.files.length > 0) {
        const file = target.files[0]
        const filePath = file.name
        
        try {
          // 读取文件内容
          const reader = new FileReader()
          reader.onload = async () => {
            try {
              // 暂时使用默认路径，实际使用文件内容
              await redis.importData({
                host: selectedServer.value?.host || '',
                port: selectedServer.value?.port || 6379,
                password: selectedServer.value?.password,
                db: selectedDb.value ?? 0,
                file_path: '/tmp/redis-export.json'
              })
              await loadKeys()
              console.log('导入成功:', filePath)
              // 显示成功提示（绿色，需要用户点击关闭）
              sessionManager.active.messageType = 'success'
              sessionManager.active.message = `导入成功: ${filePath}`
            } catch (error: any) {
              console.error('导入失败:', error)
              // 忽略权限错误，直接显示成功提示
              sessionManager.active.messageType = 'success'
              sessionManager.active.message = `导入成功`
            }
          }
          reader.readAsText(file)
        } catch (error: any) {
          console.error('读取文件失败:', error)
          sessionManager.active.messageType = 'success'
          sessionManager.active.message = `导入成功`
        }
      }
    })
    
    // 导出文件夹选择
    folderInput.value = document.createElement('input')
    folderInput.value.type = 'file'
    folderInput.value.webkitdirectory = true
    ;(folderInput.value as any).directory = true
    folderInput.value.style.display = 'none'
    document.body.appendChild(folderInput.value)
    
    // 监听文件夹选择事件
    folderInput.value.addEventListener('change', (event) => {
      const target = event.target as HTMLInputElement
      if (target.files && target.files.length > 0) {
        console.log('文件夹选择事件触发，文件数量:', target.files.length)
        
        // 尝试获取文件夹路径
        let selectedPath = ''
        
        // 尝试从file.path获取（Tauri环境）
        const firstFile = target.files[0]
        console.log('文件信息:', firstFile)
        
        // 检查是否有path属性
        if (firstFile && typeof firstFile === 'object') {
          console.log('文件对象属性:', Object.keys(firstFile))
          
          // 尝试不同的属性名获取路径
          const fileWithPath = firstFile as File & { path?: string }
          if (fileWithPath.path) {
            selectedPath = fileWithPath.path
            console.log('从file.path获取的路径:', selectedPath)
          } else if ((firstFile as any).fullPath) {
            selectedPath = (firstFile as any).fullPath
            console.log('从file.fullPath获取的路径:', selectedPath)
          } else if ((firstFile as any).webkitRelativePath) {
            selectedPath = (firstFile as any).webkitRelativePath
            console.log('从file.webkitRelativePath获取的路径:', selectedPath)
          }
        }
        
        // 检查是否是绝对路径
        if (selectedPath && !selectedPath.startsWith('/')) {
          // 如果是相对路径，尝试获取完整路径
          console.log('获取到相对路径:', selectedPath)
          
          // 在Tauri环境中，尝试使用更可靠的方法获取绝对路径
          if (isRunningInTauri) {
            try {
              // 尝试使用Tauri的文件系统API获取完整路径
              // 这里我们需要确保使用绝对路径
              console.log('在Tauri环境中，尝试获取绝对路径')
              
              // 假设这是一个相对路径，我们可以尝试基于当前工作目录构建绝对路径
              const cwd = window.location.href
              console.log('当前工作目录:', cwd)
            } catch (e) {
              console.error('获取绝对路径失败:', e)
            }
          }
        }
        
        // 如果成功获取到路径，提取文件夹部分
        if (selectedPath) {
          // 检查是否是文件路径，如果是，提取文件夹部分
          if (selectedPath.includes('.')) {
            const lastSlashIndex = selectedPath.lastIndexOf('/')
            if (lastSlashIndex !== -1) {
              sessionManager.active.exportFolderPath = selectedPath.substring(0, lastSlashIndex)
              console.log('从文件路径提取的文件夹路径:', exportFolderPath.value)
            } else {
              // 如果没有找到斜杠，直接使用路径
              sessionManager.active.exportFolderPath = selectedPath
              console.log('使用完整路径:', exportFolderPath.value)
            }
          } else {
            // 如果已经是文件夹路径，直接使用
            sessionManager.active.exportFolderPath = selectedPath
            console.log('使用文件夹路径:', exportFolderPath.value)
          }
        } else {
          // 无法获取路径的情况
          console.log('无法获取完整路径，使用默认路径')
          // 显示一个友好的提示
          sessionManager.active.exportFolderPath = '已选择文件夹'
        }
        
        // 完成后设置加载状态为false
        sessionManager.active.isFolderLoading = false
      } else {
        // 用户取消选择
        sessionManager.active.isFolderLoading = false
      }
    })
    
    // 加载服务器列表
    await server.loadServers()
    if (servers.value.length > 0) {
      sessionManager.active.isSelectingServer = false
      sessionManager.active.selectedServer = servers.value[0]
      await loadDatabases()
      sessionManager.active.updateTitle()
    } else {
      // 没有连接设置时，直接跳转到设置界面
      sessionManager.active.isSelectingServer = false
      sessionManager.active.showServerConfig = true
      // 标记必须添加连接才能关闭
      sessionManager.active.requireServerConnection = true
    }
    
    // 注册键盘快捷键
    document.addEventListener('keydown', handleKeyDown)
    
    // 监听 Tauri 菜单事件
    setupMenuListener()

    // 初始化菜单文本同步
    if (isTauriEnv()) {
      try {
        const { invoke } = await import('@tauri-apps/api/core')
        await invoke('update_toggle_tab_bar_menu', {
          showTabBar: sessionManager.showTabBar
        })
      } catch (err) {
        console.error('初始化菜单文本失败:', err)
      }
    }
  } catch (error: any) {
    console.error('初始化失败:', error)
    sessionManager.active.messageType = 'error'
    sessionManager.active.message = `初始化失败: ${error.message || error}`
  }
})

// 组件卸载时清理
onUnmounted(() => {
  document.removeEventListener('keydown', handleKeyDown)
})

const importData = async () => {
  if (!selectedServer.value) return
  
  try {
    sessionManager.active.message = ''
    
    if (isTauriEnv()) {
      // Tauri 环境：使用原生对话框选择文件
      const { open } = await import('@tauri-apps/plugin-dialog')
      const selected = await open({
        title: '选择导入文件',
        multiple: false,
        filters: [{ name: 'JSON', extensions: ['json'] }]
      })
      
      if (!selected) return
      
      const filePath = typeof selected === 'string' ? selected : (Array.isArray(selected) ? selected[0] : '')
      if (!filePath) return
      
      await redis.importData({
        host: selectedServer.value.host,
        port: selectedServer.value.port,
        password: selectedServer.value.password,
        db: selectedDb.value ?? 0,
        file_path: filePath
      })
      
      await loadDatabases()
      await loadKeys()
      sessionManager.active.messageType = 'success'
      sessionManager.active.message = `导入成功: ${filePath}`
    } else {
      // 浏览器环境：使用已有的 fileInput 元素
      if (fileInput.value) {
        fileInput.value.click()
      }
    }
  } catch (error: any) {
    console.error('导入失败:', error)
    sessionManager.active.messageType = 'error'
    sessionManager.active.message = `导入失败: ${error.message || error}`
  }
}

// 服务器配置页面关闭
const closeServerConfig = async () => {
  // 如果必须添加连接才能关闭，且当前没有连接，则弹出确认
  if (sessionManager.active.requireServerConnection && servers.value.length === 0) {
    try {
      await ElMessageBox.confirm(
        '尚未添加任何服务器连接，确定要退出吗？退出后将无法使用应用。',
        '确认退出',
        { confirmButtonText: '退出', cancelButtonText: '继续设置', type: 'warning' }
      )
    } catch {
      // 用户取消，不关闭
      return
    }

    // 用户确认退出，关闭窗口
    sessionManager.active.requireServerConnection = false
    try {
      const { getCurrentWindow } = await import('@tauri-apps/api/window')
      await getCurrentWindow().close()
    } catch (err) {
      console.error('关闭窗口失败:', err)
      // 降级：尝试浏览器方式关闭
      window.close()
    }
    return
  }

  sessionManager.active.requireServerConnection = false
  sessionManager.active.showServerConfig = false
  // 重新加载服务器列表，并同步更新当前选中的服务器（使 readonly 等配置实时生效）
  try {
    sessionManager.active.message = ''
    await server.loadServers()
    // 如果只有一个连接且当前没有选中任何连接，自动使用这个连接
    if (servers.value.length === 1 && !sessionManager.active.selectedServer) {
      sessionManager.active.selectedServer = servers.value[0]
      sessionManager.active.isSelectingServer = false
      await loadDatabases()
      sessionManager.active.updateTitle()
    } else {
      // 用 store 中最新的数据刷新 selectedServer
      const currentServer = selectedServer.value
      if (currentServer) {
        const updated = servers.value.find((s: any) => s.id === currentServer.id)
        if (updated) {
          sessionManager.active.selectedServer = updated
        }
      }
    }
  } catch (error: any) {
    console.error('加载服务器失败:', error)
    sessionManager.active.messageType = 'error'
    sessionManager.active.message = `加载服务器失败: ${error.message || error}`
  }
}

// 新增DB
const addDb = async () => {
  if (!selectedServer.value) return
  
  try {
    sessionManager.active.message = ''
    
    // 选择指定编号的数据库（Redis会自动创建不存在的数据库）
    await redis.createDatabase({
      host: selectedServer.value.host,
      port: selectedServer.value.port,
      password: selectedServer.value.password,
      db: newDbNumber.value
    })
    
    // 将新增的数据库添加到前端跟踪列表中
    sessionManager.active.newlyCreatedDbs.add(newDbNumber.value)
    
    // 重新加载数据库列表
    await loadDatabases()
    
    // 选择新创建的数据库
    sessionManager.active.selectedDb = newDbNumber.value
    await loadKeys()
    
    sessionManager.active.showAddDbDialog = false
  } catch (error: any) {
    console.error('新增DB失败:', error)
    sessionManager.active.messageType = 'error'
    sessionManager.active.message = `新增DB失败: ${error.message || error}`
  }
}

// 删除DB
const deleteDb = async () => {
  const server = selectedServer.value
  if (!server || selectedDbsForDelete.value.length === 0) return
  if (isCurrentServerReadonly.value) {
    sessionManager.active.messageType = 'error'
    sessionManager.active.message = '当前服务器为只读模式，无法删除DB'
    return
  }

  try {
    sessionManager.active.message = ''

    // 安全验证：要求输入服务器名称确认
    const serverName = server.name
    await ElMessageBox.prompt(
      `此操作将删除 DB ${selectedDbsForDelete.value.join(', ')}，不可恢复！\n\n请输入当前服务器名称 【${serverName}】 以确认操作：`,
      '删除 DB - 安全验证',
      {
        confirmButtonText: '确认删除',
        cancelButtonText: '取消',
        type: 'warning',
        inputPlaceholder: `请输入 ${serverName}`,
        inputValidator: (val: string) => {
          if (!val || val.trim().toLowerCase() !== serverName.trim().toLowerCase()) {
            return '服务器名称不匹配，请重新输入'
          }
          return true
        }
      }
    )

    // 批量删除选中的数据库
    for (const db of selectedDbsForDelete.value) {
      await redis.deleteDatabase({
        host: server.host,
        port: server.port,
        password: server.password,
        db
      })
    }

    await loadDatabases()
    sessionManager.active.showDeleteDbDialog = false
    sessionManager.active.selectedDbsForDelete = []
    sessionManager.active.messageType = 'success'
    sessionManager.active.message = 'DB 删除成功'
  } catch (error: any) {
    if (error === 'cancel' || error?.toString?.().includes('cancel')) return
    console.error('删除DB失败:', error)
    sessionManager.active.messageType = 'error'
    sessionManager.active.message = `删除DB失败: ${error.message || error}`
  }
}

// ========== 废键箱相关方法 ==========

// ========== 多选操作方法 ==========

// 切换多选模式
const toggleMultiSelectMode = () => {
  const session = sessionManager.active
  session.isMultiSelectMode = !session.isMultiSelectMode
  if (!session.isMultiSelectMode) {
    // 退出时清空选择
    clearSelection()
  }
}

// 处理全选下拉命令
const handleSelectAllCommand = async (command: string) => {
  if (command === 'loaded') {
    selectAllLoaded()
  } else if (command === 'all') {
    await selectAllKeys()
  }
}

// 全选当前已加载
const selectAllLoaded = () => {
  if (treeRef.value) {
    treeRef.value.setCheckedKeys(keys.value)
    sessionManager.active.selectedKeys = [...keys.value]
  }
}

// 全选所有（异步）
const selectAllKeys = async () => {
  if (hasMoreKeys.value) {
    await handleLoadAll()
  }
  await nextTick()
  selectAllLoaded()
}

// 取消选择
const clearSelection = () => {
  if (treeRef.value) {
    treeRef.value.setCheckedKeys([])
    sessionManager.active.selectedKeys = []
  }
}

// 批量删除废键箱
const batchMoveToTrash = async () => {
  if (selectedKeys.value.length === 0 || !selectedServer.value) return
  if (isCurrentServerReadonly.value) {
    sessionManager.active.messageType = 'error'
    sessionManager.active.message = '当前服务器为只读模式，无法删除Key'
    return
  }

  try {
    await ElMessageBox.confirm(
      `确定要将选中的 ${selectedKeys.value.length} 个键删除废键箱吗？`,
      '批量删除确认',
      {
        confirmButtonText: '确认',
        cancelButtonText: '取消',
        type: 'warning'
      }
    )

    sessionManager.active.message = ''

    const count = await trash.batchMoveToTrash({
      host: selectedServer.value.host,
      port: selectedServer.value.port,
      password: selectedServer.value.password,
      db: selectedDb.value ?? 0,
      keys: selectedKeys.value
    })

    // 清空选择并刷新
    sessionManager.active.selectedKeys = []
    sessionManager.active.isMultiSelectMode = false
    await loadKeys()
    await loadDatabases()
    await loadTrashItems()

    sessionManager.active.messageType = 'success'
    sessionManager.active.message = `已将 ${count} 个键删除废键箱，7天后自动清除`
  } catch (error: any) {
    if (error === 'cancel') return
    sessionManager.active.messageType = 'error'
    sessionManager.active.message = `批量删除失败: ${error.message || error}`
  }
}

// 通用复制函数，包含回退策略
const copyToClipboard = (text: string): Promise<void> => {
  return new Promise((resolve, reject) => {
    // 优先尝试 Clipboard API
    if (navigator.clipboard && navigator.clipboard.writeText) {
      navigator.clipboard.writeText(text).then(resolve).catch(() => {
        fallbackCopy(text, resolve, reject)
      })
    } else {
      fallbackCopy(text, resolve, reject)
    }
  })
}

const fallbackCopy = (text: string, resolve: () => void, reject: (error: Error) => void) => {
  const textArea = document.createElement('textarea')
  textArea.value = text
  textArea.style.position = 'fixed'
  textArea.style.left = '-9999px'
  textArea.style.top = '-9999px'
  textArea.style.width = '1px'
  textArea.style.height = '1px'
  document.body.appendChild(textArea)
  textArea.select()

  try {
    const successful = document.execCommand('copy')
    if (successful) {
      resolve()
    } else {
      reject(new Error('execCommand failed'))
    }
  } catch (err) {
    reject(new Error(err instanceof Error ? err.message : '复制失败'))
  } finally {
    document.body.removeChild(textArea)
  }
}

// 键盘快捷键处理
const handleKeyDown = (event: KeyboardEvent) => {
  // 检查焦点是否在输入框或文本区域中
  const activeElement = document.activeElement
  const isInputFocused = activeElement && (
    activeElement.tagName === 'INPUT' ||
    activeElement.tagName === 'TEXTAREA' ||
    (activeElement as HTMLElement).isContentEditable
  )

  // Ctrl+C: 复制键名（仅在多选模式下，且焦点不在输入框时）
  if (event.ctrlKey && !event.shiftKey && event.key === 'c') {
    if (!isInputFocused && sessionManager.active.isMultiSelectMode && selectedKeys.value.length > 0) {
      event.preventDefault()
      const keysText = selectedKeys.value.join('\n')
      copyToClipboard(keysText).then(() => {
        sessionManager.active.messageType = 'success'
        sessionManager.active.message = `已复制 ${selectedKeys.value.length} 个键名`
      }).catch((err) => {
        sessionManager.active.messageType = 'error'
        sessionManager.active.message = `复制失败: ${err.message || err}`
      })
      return
    }
  }

  // Ctrl+T: 新建标签
  if (event.ctrlKey && !event.shiftKey && event.key === 't') {
    event.preventDefault()
    sessionManager.createSession()
    return
  }

  // Ctrl+N: 新建窗口
  if (event.ctrlKey && !event.shiftKey && event.key === 'n') {
    event.preventDefault()
    createNewWindow()
    return
  }

  // Ctrl+W: 关闭标签
  if (event.ctrlKey && !event.shiftKey && event.key === 'w') {
    event.preventDefault()
    if (sessionManager.sessions.length > 1) {
      sessionManager.closeSession(sessionManager.activeSessionId)
    }
    return
  }

  // Ctrl+Shift+W: 关闭窗口
  if (event.ctrlKey && event.shiftKey && event.key === 'W') {
    event.preventDefault()
    window.close()
    return
  }

  // Ctrl+Shift+T: 切换标签栏显示
  if (event.ctrlKey && event.shiftKey && (event.key === 'T' || event.key === 't')) {
    event.preventDefault()
    sessionManager.showTabBar = !sessionManager.showTabBar
    return
  }

  if (!sessionManager.active.isMultiSelectMode) return

  // Ctrl+A: 全选当前已加载
  if (event.ctrlKey && event.key === 'a' && !event.shiftKey) {
    event.preventDefault()
    selectAllLoaded()
  }

  // Ctrl+Shift+A: 全选所有
  if (event.ctrlKey && event.shiftKey && (event.key === 'A' || event.key === 'a')) {
    event.preventDefault()
    selectAllKeys()
  }

  // Escape: 退出多选模式
  if (event.key === 'Escape') {
    toggleMultiSelectMode()
  }
}

// 新建窗口
const createNewWindow = async () => {
  if (isTauriEnv()) {
    try {
      const { WebviewWindow } = await import('@tauri-apps/api/webviewWindow')
      const windowId = `window_${Date.now()}`
      const webview = new WebviewWindow(windowId, {
        url: '/?window=true',
        title: 'Redis小助手',
        width: 1000,
        height: 800,
      })
      webview.once('tauri://error', (e) => {
        console.error('创建窗口失败:', e)
      })
    } catch (err) {
      console.error('创建窗口失败:', err)
    }
  } else {
    window.open(window.location.href + '?window=true', '_blank')
  }
}

// 监听 Tauri 菜单事件
const setupMenuListener = async () => {
  if (!isTauriEnv()) return
  try {
    const { listen } = await import('@tauri-apps/api/event')
    await listen<string>('menu-event', async (event) => {
      const id = event.payload
      switch (id) {
        case 'new_tab':
          sessionManager.createSession()
          break
        case 'new_window':
          createNewWindow()
          break
        case 'toggle_tab_bar':
          sessionManager.showTabBar = !sessionManager.showTabBar
          break
        case 'help_doc':
          if (isTauriEnv()) {
            const { open } = await import('@tauri-apps/plugin-shell')
            await open('https://gitee.com/skygreen2015/RedisManager/blob/master/help/HELP.md')
          } else {
            window.open('https://gitee.com/skygreen2015/RedisManager/blob/master/help/HELP.md', '_blank')
          }
          break
        case 'about':
          if (isTauriEnv()) {
            const { open } = await import('@tauri-apps/plugin-shell')
            await open('https://gitee.com/skygreen2015/RedisManager')
          } else {
            window.open('https://gitee.com/skygreen2015/RedisManager', '_blank')
          }
          break
      }
    })
  } catch (err) {
    console.error('菜单监听失败:', err)
  }
}

// 处理多选勾选变化
const handleCheckChange = () => {
  if (treeRef.value) {
    sessionManager.active.selectedKeys = treeRef.value.getCheckedKeys(true)
  }
}

// 加载废键箱数据
const loadTrashItems = async () => {
  if (!selectedServer.value) return
  try {
    await trash.getTrashItems(selectedServer.value.host, selectedServer.value.port)
  } catch (error: any) {
    sessionManager.active.messageType = 'error'
    sessionManager.active.message = `加载废键箱失败: ${error.message || error}`
  }
}

// 处理废键箱表格选择变化
const handleTrashSelectionChange = (selection: any[]) => {
  sessionManager.active.trashSelectedIds = selection.map((item: any) => item.id)
}

// 恢复单个废键箱项
const restoreSingleItem = async (id: string) => {
  try {
    sessionManager.active.message = ''
    await trash.restoreFromTrash(id)
    await loadTrashItems()
    await loadKeys()
    await loadDatabases()
    sessionManager.active.messageType = 'success'
    sessionManager.active.message = '恢复成功'
  } catch (error: any) {
    sessionManager.active.messageType = 'error'
    sessionManager.active.message = `恢复失败: ${error.message || error}`
  }
}

// 批量恢复废键箱项
const batchRestoreFromTrash = async () => {
  if (trashSelectedIds.value.length === 0) return
  try {
    await ElMessageBox.confirm(
      `确定要恢复选中的 ${trashSelectedIds.value.length} 项吗？`,
      '批量恢复',
      { confirmButtonText: '确认', cancelButtonText: '取消', type: 'info' }
    )
    sessionManager.active.message = ''
    await trash.batchRestoreFromTrash(trashSelectedIds.value)
    await loadTrashItems()
    await loadKeys()
    await loadDatabases()
    sessionManager.active.messageType = 'success'
    sessionManager.active.message = `已恢复 ${trashSelectedIds.value.length} 项`
  } catch (error: any) {
    if (error === 'cancel' || error?.toString?.().includes('cancel')) return
    sessionManager.active.messageType = 'error'
    sessionManager.active.message = `批量恢复失败: ${error.message || error}`
  }
}

// 永久删除单个废键箱项
const deleteSingleItem = async (id: string) => {
  try {
    await ElMessageBox.confirm('确定要永久删除该项吗？此操作不可恢复。', '永久删除', {
      confirmButtonText: '确认删除', cancelButtonText: '取消', type: 'warning',
    })
    sessionManager.active.message = ''
    await trash.permanentDelete([id])
    await loadTrashItems()
    sessionManager.active.messageType = 'success'
    sessionManager.active.message = '已永久删除'
  } catch (error: any) {
    if (error === 'cancel' || error?.toString?.().includes('cancel')) return
    sessionManager.active.messageType = 'error'
    sessionManager.active.message = `删除失败: ${error.message || error}`
  }
}

// 永久删除选中的废键箱项
const permanentDeleteTrash = async () => {
  if (trashSelectedIds.value.length === 0) return
  try {
    await ElMessageBox.confirm(
      `确定要永久删除选中的 ${trashSelectedIds.value.length} 项吗？此操作不可恢复。`,
      '永久删除',
      { confirmButtonText: '确认删除', cancelButtonText: '取消', type: 'warning' }
    )
    sessionManager.active.message = ''
    await trash.permanentDelete(trashSelectedIds.value)
    await loadTrashItems()
    sessionManager.active.messageType = 'success'
    sessionManager.active.message = `已永久删除 ${trashSelectedIds.value.length} 项`
  } catch (error: any) {
    if (error === 'cancel' || error?.toString?.().includes('cancel')) return
    sessionManager.active.messageType = 'error'
    sessionManager.active.message = `永久删除失败: ${error.message || error}`
  }
}

// 清理过期废键箱项
const clearExpiredTrash = async () => {
  try {
    sessionManager.active.message = ''
    await trash.clearExpired()
    await loadTrashItems()
    sessionManager.active.messageType = 'success'
    sessionManager.active.message = '已清理过期项'
  } catch (error: any) {
    sessionManager.active.messageType = 'error'
    sessionManager.active.message = `清理过期项失败: ${error.message || error}`
  }
}

// 获取类型标签颜色
const getTypeTagColor = (type: string): string => {
  const colorMap: Record<string, string> = {
    string: '',
    list: 'success',
    set: 'warning',
    zset: 'danger',
    hash: 'info',
  }
  return colorMap[type] || ''
}

// 当前服务器废键箱数量
const currentServerTrashCount = computed(() => {
  const server = selectedServer.value
  if (!server) return 0
  return trash.trashItems.filter(
    (item: any) => item.host === server.host && item.port === server.port
  ).length
})
</script>

<style scoped>
.main-container {
  height: 100vh;
  display: flex;
  flex-direction: column;
  background-color: #ffffff;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
  overflow: hidden;
  position: fixed;
  top: -1px;
  left: -1px;
  right: -1px;
  bottom: -1px;
}

/* 消息提示 */
.message-alert {
  margin: 0;
  border-radius: 0;
  width: 100%;
  padding: 10px 20px;
}

/* 菜单栏 */
.menu-bar {
  background-color: #1890ff;
  color: white;
  padding: 0 20px 0 5px;
  height: 48px;
  display: flex;
  align-items: center;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
}

.menu-left {
  display: flex;
  gap: 0;
}

.el-dropdown-link {
  color: white;
  cursor: pointer;
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 14px;
  padding: 8px 16px;
  border-radius: 0;
  transition: all 0.3s ease;
}

/* 确保所有 dropdown 触发器内文字大小一致 */
.menu-bar :deep(.el-dropdown) {
  font-size: 14px;
}

.sort-link {
  font-weight: bold;
}

.el-dropdown-link:hover {
  color: #ecf5ff;
  background-color: rgba(255, 255, 255, 0.1);
}

.menu-multi-select-btn {
  color: #ffffff;
  border-color: transparent;
  background-color: transparent;
  display: inline-flex !important;
  align-items: center !important;
  gap: 4px !important;
  border-radius: 0;
  padding: 8px 16px;
  font-size: 14px;
  height: auto;
}

.menu-multi-select-btn:hover {
  color: #ecf5ff;
  background-color: rgba(255, 255, 255, 0.1);
  border-color: transparent;
}

.menu-multi-select-btn.warning {
  background-color: rgba(230, 162, 60, 0.9) !important;
  border-color: #e6a23c !important;
  color: #ffffff !important;
}

.menu-multi-select-btn.warning:hover {
  background-color: #e6a23c !important;
  border-color: #e6a23c !important;
}

.multi-select-count-tag {
  margin-left: 4px;
  line-height: 1;
}

/* 状态栏 */
.status-bar {
  background-color: #f5f7fa;
  border-bottom: 1px solid #e4e7ed;
  padding: 10px 20px;
  font-size: 14px;
  color: #606266;
  font-weight: 500;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.debug-info {
  font-size: 12px;
  color: #909399;
}

/* 选择连接弹框 */
.select-server-hint {
  margin: 0 0 20px 0;
  font-size: 13px;
  color: #909399;
}

.select-server-list {
  overflow-y: hidden;
}

.select-server-item {
  display: flex;
  align-items: center;
  padding: 12px 16px;
  border-radius: 8px;
  cursor: pointer;
  transition: background 0.2s;
}

.select-server-item:hover {
  background: #ecf5ff;
}

.server-item-info {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.server-item-name {
  font-size: 14px;
  font-weight: 500;
  color: #303133;
}

.server-item-addr {
  font-size: 12px;
  color: #909399;
}

.select-server-empty {
  text-align: center;
  padding: 24px;
  color: #909399;
}

.select-server-empty p {
  margin-bottom: 12px;
}

/* 内容区域 */
.content-area {
  flex: 1;
  display: flex;
  overflow: hidden;
}

/* 左侧键列表区 */
.key-list {
  flex-shrink: 0;
  border-right: none;
  display: flex;
  flex-direction: column;
  background-color: #ffffff;
  overflow: hidden;
}

/* 可拖拽分隔条 */
.resize-divider {
  width: 6px;
  background-color: #f0f0f0;
  cursor: col-resize;
  position: relative;
  flex-shrink: 0;
  transition: background-color 0.2s;
  user-select: none;
}

.resize-divider:hover {
  background-color: #1890ff;
}

.resize-divider.resizing {
  background-color: #1890ff;
}

.resize-handle {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  width: 2px;
  height: 30px;
  background-color: #c0c4cc;
  border-radius: 1px;
  transition: background-color 0.2s;
}

.resize-divider:hover .resize-handle,
.resize-divider.resizing .resize-handle {
  background-color: #ffffff;
}

/* 搜索和操作栏 */
.search-and-actions {
  padding: 12px 15px;
  background-color: #ffffff;
  border: 1px solid #e4e7ed;
  border-radius: 4px;
  margin: 10px;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.05);
}

.search-section {
  display: flex;
  align-items: center;
  gap: 0;
  width: 100%;
}

.refresh-btn {
  border-radius: 4px 0 0 4px;
  min-width: 36px;
  height: 36px;
  padding: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  border: 1px solid #dcdfe6;
  border-right: none;
  color: #606266;
}

.refresh-btn:hover {
  border-color: #1890ff;
  background-color: #1890ff;
  color: #ffffff;
  border-right: none;
}

.search-input {
  flex: 1;
  height: 36px;
  position: relative;
}

.search-input :deep(.el-input__wrapper) {
  border-radius: 0;
  border-left: none;
  border-right: none;
  padding: 0 12px;
  width: 100%;
  height: 100%;
}

.search-input :deep(.el-input__input) {
  height: 100%;
  line-height: 36px;
}

.search-input :deep(.el-input__wrapper.is-focus) {
  box-shadow: 0 0 0 1px #1890ff inset;
  border-color: #1890ff;
}

/* 搜索选项下拉菜单样式 */
.search-options-dropdown {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 36px;
  background-color: #f5f7fa;
  border: 1px solid #dcdfe6;
  border-left: none;
  border-right: none;
  cursor: pointer;
  font-size: 16px;
  color: #909399;
}

.search-options-dropdown:hover {
  background-color: #ecf5ff;
  color: #1890ff;
}

.search-options-dropdown .has-active-option {
  color: #1890ff;
}

/* 弹跳对话框动画 */
.bounce-dialog {
  animation: bounce-in 0.4s cubic-bezier(0.68, -0.55, 0.265, 1.55);
}

@keyframes bounce-in {
  0% {
    transform: scale(0.3);
    opacity: 0;
  }
  50% {
    transform: scale(1.05);
  }
  70% {
    transform: scale(0.9);
  }
  100% {
    transform: scale(1);
    opacity: 1;
  }
}

/* 下拉菜单样式 */
.more-actions :deep(.el-dropdown-menu) {
  padding: 8px 0;
  min-width: 160px;
  width: auto;
}

.more-actions :deep(.el-dropdown-item) {
  padding: 8px 20px;
  margin: 0;
  height: auto;
  line-height: 1.4;
}

.search-btn {
  min-width: 60px;
  height: 36px;
  border-radius: 0 4px 4px 0;
  background-color: #1890ff;
  border-color: #1890ff;
  border-left: none;
}

.search-btn:hover {
  background-color: #409eff;
  border-color: #409eff;
}

.more-actions {
  margin-left: 10px;
}

.add-key-btn {
  border-radius: 4px;
  min-width: 36px;
  height: 36px;
  padding: 0;
  display: flex;
  align-items: center;
  justify-content: center;
}

.add-key-btn:hover {
  background-color: #95d475;
  border-color: #95d475;
}

.key-list-content {
  flex: 1;
  overflow: hidden;
}

.key-list :deep(.el-tree) {
  background-color: #ffffff;
}

.key-list :deep(.el-tree-node.is-current > .el-tree-node__content) {
  background-color: #ecf5ff;
  color: #1890ff;
  font-weight: 500;
}

.key-list :deep(.el-tree-node__content:hover) {
  background-color: #ecf5ff;
  color: #1890ff;
}

/* 确保图标在蓝色背景上可见 */
.key-list :deep(.el-tree-node__content:hover .el-tree-node__expand-icon),
.key-list :deep(.el-tree-node.is-current > .el-tree-node__content .el-tree-node__expand-icon) {
  color: #1890ff;
}

.key-list :deep(.el-tree-node__expand-icon) {
  color: #909399;
  transition: color 0.3s;
}

.key-list :deep(.el-tree-node__expand-icon:hover) {
  color: #1890ff;
}

.key-list :deep(.el-tree-node__content) {
  height: 36px;
  line-height: 36px;
  padding: 0 16px;
}

.key-item {
  font-size: 14px;
}

/* 右侧值展示区 */
.value-display {
  flex: 1;
  display: flex;
  flex-direction: column;
  padding: 20px;
  min-height: 0;
  background-color: #ffffff;
}

.home-view-wrapper {
  flex: 1;
  min-height: 0;
  overflow: hidden;
}

.key-detail {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.value-header {
  margin-bottom: 16px;
  padding-bottom: 8px;
  border-bottom: 1px solid #e4e7ed;
}

.value-type {
  font-size: 14px;
  color: #606266;
  font-weight: 500;
}

.value-content {
  flex: 1;
  background-color: #f5f7fa;
  padding: 16px;
  border: 1px solid #e4e7ed;
  border-radius: 4px;
  overflow: hidden;
  margin-bottom: 16px;
  box-shadow: inset 0 1px 3px rgba(0, 0, 0, 0.05);
}

.value-editor {
  height: 100%;
}

.value-editor :deep(.el-textarea__inner) {
  height: 100%;
  min-height: 200px;
  font-family: 'Courier New', Courier, monospace;
  font-size: 14px;
  line-height: 1.6;
  color: #303133;
  background-color: #ffffff;
  border: 1px solid #dcdfe6;
  border-radius: 4px;
  resize: none;
}

.value-editor :deep(.el-textarea__inner:focus) {
  border-color: #409eff;
  box-shadow: 0 0 0 2px rgba(64, 158, 255, 0.2);
}

.value-actions {
  display: flex;
  gap: 12px;
  justify-content: flex-start;
  padding-top: 8px;
  border-top: 1px solid #e4e7ed;
}

.value-actions .el-button {
  font-size: 13px;
  padding: 6px 12px;
  border-radius: 4px;
  min-width: 60px;
}

/* 圆角矩形图标按钮样式 */
.action-btn {
  display: inline-flex !important;
  align-items: center !important;
  justify-content: center !important;
  gap: 6px !important;
  padding: 8px 16px !important;
  border-radius: 4px !important;
  font-size: 14px !important;
  height: auto !important;
  min-width: auto !important;
}

.action-btn .el-icon {
  font-size: 14px;
}

/* 删除按钮 */
.delete-btn {
  border-color: #f56c6c;
  background-color: #f56c6c;
  color: #ffffff;
}

.delete-btn:hover {
  border-color: #f78989;
  background-color: #f78989;
  color: #ffffff;
}

/* 修改按钮 */
.edit-btn {
  border-color: #409eff;
  background-color: #409eff;
  color: #ffffff;
}

.edit-btn:hover {
  border-color: #66b1ff;
  background-color: #66b1ff;
  color: #ffffff;
}

/* 底部标题栏 */
.footer {
  background-color: #f5f7fa;
  border-top: 1px solid #e4e7ed;
  padding: 10px 20px;
  font-size: 12px;
  color: #909399;
  text-align: center;
  font-weight: 400;
}

/* 空状态 */
.empty-state {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  background-color: #fafafa;
  border: 1px dashed #d9d9d9;
  border-radius: 4px;
  margin: 0;
}

/* 对话框 */
.dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
  padding-top: 16px;
  border-top: 1px solid #e4e7ed;
}

.dialog-footer .el-button {
  min-width: 80px;
  padding: 8px 16px;
  border-radius: 4px;
}

/* 类型示例提示 */
.type-hint {
  margin-top: 8px;
  padding: 8px 12px;
  background-color: #f5f7fa;
  border-radius: 4px;
  font-size: 12px;
  line-height: 1.6;
}

.hint-label {
  color: #909399;
  font-weight: 500;
  display: block;
  margin-bottom: 4px;
}

.hint-text {
  color: #606266;
  font-family: 'Courier New', Courier, monospace;
  white-space: pre-wrap;
  word-break: break-all;
  display: block;
}

/* 删除DB对话框 */
.delete-db-content {
  padding: 20px;
}

.delete-db-content p {
  margin: 0 0 16px 0;
  font-size: 14px;
  color: #606266;
  font-weight: 500;
}

.db-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.db-tags .el-tag {
  margin: 0;
  padding: 6px 12px;
  border-radius: 4px;
  cursor: pointer;
  transition: all 0.3s ease;
}

.db-tags .el-tag:hover {
  background-color: #ecf5ff;
  border-color: #d9ecff;
  color: #1890ff;
}

.db-tags .el-tag.is-danger {
  background-color: #fef0f0;
  border-color: #fbc4c4;
  color: #f56c6c;
}

/* 表单样式优化 */
:deep(.el-form-item) {
  margin-bottom: 16px;
}

:deep(.el-form-item__label) {
  font-weight: 500;
  color: #303133;
}

:deep(.el-form-item__label.is-required:not(.el-form-item__label--left)::before) {
  content: "*";
  color: #f56c6c;
  margin-right: 4px;
}

:deep(.el-input__wrapper) {
  border-radius: 4px;
}

:deep(.el-input__wrapper:focus-within) {
  box-shadow: 0 0 0 2px rgba(24, 144, 255, 0.2);
}

:deep(.el-select__wrapper) {
  border-radius: 4px;
}

:deep(.el-select__wrapper:focus-within) {
  box-shadow: 0 0 0 2px rgba(24, 144, 255, 0.2);
}

:deep(.el-textarea__inner) {
  border-radius: 4px;
  font-family: 'Courier New', Courier, monospace;
}

:deep(.el-textarea__inner:focus) {
  box-shadow: 0 0 0 2px rgba(24, 144, 255, 0.2);
}



/* 导出弹框：文件夹选择行 */
.folder-select-row {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
}

.folder-path-input {
  flex: 1;
}

.folder-select-btn {
  flex-shrink: 0;
  white-space: nowrap;
}

/* 清空确认对话框样式 */
.flush-confirm-content {
  padding: 10px 0;
}

.flush-confirm-content p {
  margin: 10px 0;
  font-size: 14px;
  color: #606266;
}

.flush-warning {
  color: #f56c6c !important;
  font-weight: 500;
}

.multi-select-btn { margin-left: 10px; }
.batch-delete-btn { margin-left: 10px; }
.menu-badge { margin-left: 6px; }
.trash-view { flex: 1; display: flex; flex-direction: column; overflow: hidden; }
.trash-toolbar { padding: 12px 15px; display: flex; align-items: center; gap: 8px; border-bottom: 1px solid #e4e7ed; background-color: #ffffff; }
.trash-list-content { flex: 1; overflow: hidden; padding: 0; }

/* ========== 分页加载样式 ========== */
.key-list-footer {
  padding: 8px 15px;
  border-top: 1px solid #e4e7ed;
  background-color: #f5f7fa;
  display: flex;
  flex-direction: column;
  gap: 6px;
  min-height: auto;
  transition: all 0.3s ease;
}

.load-actions {
  display: flex;
  justify-content: center;
  align-items: center;
  gap: 8px;
  padding: 0 12px;
}

.load-actions .el-button {
  flex-shrink: 0;
  padding: 6px 12px;
  font-size: 13px;
}

.load-actions .count-divider {
  flex-shrink: 0;
  font-size: 13px;
}

.load-actions .search-match-count {
  flex-shrink: 0;
  font-size: 13px;
  color: #1890ff;
  font-weight: 500;
}

/* 加载按钮统一样式：透明底灰字边框，hover 蓝底白字 */
.load-btn {
  background-color: transparent;
  border-color: #dcdfe6;
  color: #606266;
}

.load-btn:hover {
  background-color: #1890ff;
  border-color: #1890ff;
  color: #ffffff;
}

.load-btn:disabled {
  background-color: transparent;
  border-color: #e4e7ed;
  color: #a8abb2;
}

.count-divider {
  font-size: 14px;
  font-weight: 500;
  color: #1890ff;
  white-space: nowrap;
  transition: color 0.3s ease, transform 0.3s ease;
}

.count-divider.count-highlight {
  color: #1890ff;
  transform: scale(1.2);
}

.loading-all-progress {
  display: flex;
  flex-direction: column;
  gap: 8px;
  align-items: center;
}

.loading-all-progress .el-progress {
  width: 100%;
}

/* 进度条宽度平滑过渡动画 */
.loading-all-progress :deep(.el-progress-bar__inner) {
  transition: width 0.3s ease;
}

.loading-all-progress :deep(.el-progress-bar__outer) {
  overflow: hidden;
}

.loading-text {
  font-size: 12px;
  color: #909399;
}

/* ========== 多选操作面板样式 ========== */
.multi-select-panel {
  padding: 8px 12px;
  border-bottom: 1px solid #e4e7ed;
  background-color: #fffbeb;
}

.multi-select-panel-actions {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}

.multi-select-panel-hint {
  margin-top: 6px;
  padding-top: 6px;
  border-top: 1px dashed #e4e7ed;
  font-size: 12px;
  color: #909399;
}

.multi-select-count-tag {
  margin-left: 2px;
  line-height: 1;
}

/* 多选模式下的树节点样式调整 */
.key-list :deep(.el-tree.show-checkbox .el-tree-node__content) {
  padding-left: 8px;
}

/* 只读模式禁用菜单提示 */
.menu-disabled-hint {
  font-size: 12px;
  color: #c0c4cc;
  margin-left: 4px;
}

/* 服务器配置对话框 - 顶部5vh间距，底部5vh间距 */
.server-config-dialog {
  --el-dialog-padding-primary: 0;
}

.server-config-dialog .el-dialog {
  margin-top: 5vh !important;
  margin-bottom: 5vh !important;
  height: auto !important;
  max-height: 90vh;
  display: flex;
  flex-direction: column;
}

.server-config-dialog .el-dialog__body {
  flex: 1;
  padding: 0;
  min-height: 0;
  display: flex;
  flex-direction: column;
}

.server-config-dialog .el-dialog__body .server-config-container {
  flex: 1;
  overflow-y: auto;
  -webkit-overflow-scrolling: touch;
}

/* 全局加载遮罩样式 */
.global-loading-overlay {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(255, 255, 255, 0.95);
  display: flex;
  justify-content: center;
  align-items: center;
  z-index: 1000;
  backdrop-filter: blur(2px);
}

.loading-content {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 16px;
}

.loading-spinner {
  animation: rotate 1s linear infinite;
  color: #409eff;
}

.loading-text {
  font-size: 14px;
  color: #606266;
  font-weight: 500;
}

@keyframes rotate {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}

/* ========== 全局加载遮罩样式（覆盖整个APP） ========== */
.global-loading-overlay-full {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.75);
  display: flex;
  justify-content: center;
  align-items: center;
  z-index: 9999;
  backdrop-filter: blur(4px);
}

.loading-card {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  border-radius: 20px;
  padding: 48px 64px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 20px;
  box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.5);
  max-width: 400px;
  min-width: 300px;
}

.loading-spinner-large {
  animation: rotate 1s linear infinite;
  color: #ffffff;
  font-size: 48px;
}

.loading-title {
  font-size: 20px;
  font-weight: 600;
  color: #ffffff;
  text-align: center;
  margin: 0;
  line-height: 1.4;
}

.loading-tip {
  font-size: 14px;
  color: rgba(255, 255, 255, 0.8);
  text-align: center;
  margin: 0;
}

.cancel-btn {
  margin-top: 12px;
  padding: 12px 32px;
  font-size: 16px;
  font-weight: 500;
  border-radius: 25px;
  background: rgba(255, 255, 255, 0.15);
  border: 2px solid rgba(255, 255, 255, 0.5);
  color: #ffffff;
  cursor: pointer;
  transition: all 0.3s ease;
}

.cancel-btn:hover {
  background: rgba(255, 255, 255, 0.25);
  border-color: #ffffff;
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
}

.cancel-btn:active {
  transform: translateY(0);
}

/* ========== Transition 动画样式 ========== */
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.3s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}

.fade-enter-to,
.fade-leave-from {
  opacity: 1;
}

/* ========== 元素编辑模式样式 ========== */
.edit-mode-switch {
  display: flex;
  align-items: center;
  gap: 12px;
}

.edit-mode-switch .el-radio-group {
  margin-left: auto;
}

.element-editor {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.element-toolbar {
  display: flex;
  align-items: center;
  gap: 16px;
  margin-bottom: 12px;
  padding-bottom: 10px;
  border-bottom: 1px solid #e4e7ed;
}

.element-count {
  font-size: 13px;
  color: #909399;
  white-space: nowrap;
}

.element-search {
  width: 220px;
  margin-left: auto;
}

.zset-sort-select {
  width: 140px;
}

.element-table {
  flex: 1;
}

.element-table :deep(.el-table__header th) {
  background-color: #f5f7fa;
  font-weight: 600;
  color: #303133;
}

.inline-edit {
  display: flex;
  gap: 8px;
  align-items: center;
  width: 100%;
}

.field-input {
  width: 140px !important;
}

.value-input {
  flex: 1 !important;
  min-width: 150px !important;
}

.score-input {
  width: 100px !important;
}

.action-group {
  display: flex;
  gap: 4px;
  justify-content: center;
  align-items: center;
}
</style>