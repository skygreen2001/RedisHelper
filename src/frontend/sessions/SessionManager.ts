/**
 * SessionManager - 标签管理器
 * 每个窗口拥有独立的 SessionManager 实例
 */
import { reactive } from 'vue'
import { Session, ServerConfig } from './Session'

const SHOW_TAB_BAR_KEY = 'redis-helper-show-tab-bar'

export class SessionManagerClass {
  sessions: Session[] = reactive([])
  activeSessionId: string = ''
  _showTabBar = false

  constructor() {
    // 初始化时从 localStorage 读取设置
    this.loadShowTabBar()
    // 监听 storage 事件，实现跨窗口同步
    if (typeof window !== 'undefined') {
      window.addEventListener('storage', (event) => {
        if (event.key === SHOW_TAB_BAR_KEY) {
          this._showTabBar = event.newValue === 'true'
          this.syncMenuText(this._showTabBar)
        }
      })
    }
  }

  /** 从 localStorage 加载 showTabBar 设置 */
  private loadShowTabBar(): void {
    try {
      const saved = localStorage.getItem(SHOW_TAB_BAR_KEY)
      this._showTabBar = saved === 'true'
    } catch {
      this._showTabBar = false
    }
  }

  /** 保存 showTabBar 设置到 localStorage */
  private saveShowTabBar(): void {
    try {
      localStorage.setItem(SHOW_TAB_BAR_KEY, String(this._showTabBar))
    } catch (err) {
      console.error('保存标签栏设置失败:', err)
    }
  }

  /** 获取标签栏显示状态（全局） */
  get showTabBar(): boolean {
    return this._showTabBar
  }

  /** 设置标签栏显示状态（全局） */
  set showTabBar(value: boolean) {
    const oldValue = this._showTabBar
    this._showTabBar = value

    // 同步更新菜单文本
    if (oldValue !== value) {
      this.saveShowTabBar()
      this.syncMenuText(value)
    }
  }

  /** 同步菜单文本到 Rust 后端 */
  private async syncMenuText(showTabBar: boolean): Promise<void> {
    if (typeof window === 'undefined') return
    const w = window as any
    if (!w.__TAURI__ && !w.__TAURI_INTERNALS__ && !w.__TAURI_IPC__) return

    try {
      const { invoke } = await import('@tauri-apps/api/core')
      await invoke('update_toggle_tab_bar_menu', { showTabBar })
    } catch (err) {
      console.error('同步菜单文本失败:', err)
    }
  }

  /** 获取当前活动会话 */
  get active(): Session {
    if (this.sessions.length === 0) {
      this.createSession('新标签')
    }
    return this.sessions.find(s => s.id === this.activeSessionId) || this.sessions[0]
  }

  /** 是否为独立窗口模式 */
  get isWindowMode(): boolean {
    return new URLSearchParams(window.location.search).has('window')
  }

  /** 创建新会话 */
  createSession(title?: string, server?: ServerConfig, activate: boolean = true): Session {
    const session = reactive(new Session(title || '新标签')) as Session
    if (server) {
      session.selectedServer = server
      session.updateTitle()
    } else {
      // 没有指定服务器时，需要用户选择连接
      session.isSelectingServer = true
    }
    this.sessions.push(session)
    if (activate) {
      this.activeSessionId = session.id
    }
    return session
  }

  /** 关闭会话 */
  closeSession(id: string) {
    const idx = this.sessions.findIndex(s => s.id === id)
    if (idx < 0) return
    if (this.sessions.length <= 1) return

    this.sessions.splice(idx, 1)

    if (this.activeSessionId === id) {
      const newIdx = Math.min(idx, this.sessions.length - 1)
      this.activeSessionId = this.sessions[newIdx].id
    }
  }

  /** 切换活动会话 */
  switchSession(id: string) {
    if (this.sessions.some(s => s.id === id)) {
      this.activeSessionId = id
    }
  }

  /** 初始化 */
  init() {
    if (this.sessions.length === 0) {
      this.createSession('新标签')
    }
    if (!this.activeSessionId && this.sessions.length > 0) {
      this.activeSessionId = this.sessions[0].id
    }
  }
}

// 每个窗口独立的实例（通过 window 属性缓存）
function getSessionManager(): SessionManagerClass {
  const w = window as any
  if (!w.__sessionManager__) {
    w.__sessionManager__ = reactive(new SessionManagerClass()) as SessionManagerClass
  }
  return w.__sessionManager__
}

export const sessionManager = getSessionManager()
