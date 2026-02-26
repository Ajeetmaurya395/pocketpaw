# Multi-Mode Workspace Client — Implementation Plan

**Date:** 2026-02-26
**Design doc:** `docs/plans/2026-02-26-multi-mode-workspace-client-design.md`
**Stack:** Tauri 2.0 + SvelteKit 2 + Svelte 5 runes + Tailwind CSS 4

---

## Phase 1: Foundation (Tab System + Tiling Layout + Local File Explorer)

### 1.1 — Install new frontend dependencies

```bash
cd client && bun add monaco-editor @xterm/xterm @xterm/addon-fit pdfjs-dist xlsx diff2html marked
```

**Files:** `client/package.json`

### 1.2 — Create `workspaceStore`

New store managing tabs, layout trees, and active state.

**File:** `client/src/lib/stores/workspace.svelte.ts`

```typescript
// Key types
interface Tab {
  id: string
  type: "chat" | "workspace"
  title: string
  sessionId: string
  layoutTree: LayoutNode | null
  openFiles: OpenFile[]
}

interface LayoutNode {
  id: string
  type: "split" | "widget" | "tabs"
  direction?: "horizontal" | "vertical"
  ratio?: number
  children?: LayoutNode[]
  widgetId?: string
  activeTabIndex?: number  // for "tabs" type
}

interface OpenFile {
  path: string
  provider: "local" | "remote" | "cloud"
  isDirty: boolean
}

// State
- tabs: Tab[] (default: one chat tab)
- activeTabId: string
- layoutPresets: Record<string, LayoutNode> (built-in presets)

// Methods
- addTab(type, title?): string — creates tab, returns id
- closeTab(id): void — closes tab, handles last-tab
- switchTab(id): void
- reorderTabs(fromIdx, toIdx): void
- convertToWorkspace(tabId): void — chat → workspace with default layout
- updateLayout(tabId, newTree): void
- savePreset(name, tree): void
- loadPreset(name): LayoutNode

// Persistence: save/load to localStorage on change
```

**Also update:** `client/src/lib/stores/index.ts` to export workspace store.

### 1.3 — Create `TabBar.svelte` component

Horizontal tab bar below the title bar.

**File:** `client/src/lib/components/workspace/TabBar.svelte`

- Renders each tab with icon (💬 or 📁), title, close button
- "+" button to create new tab
- Middle-click to close
- Drag to reorder (use HTML5 drag-and-drop)
- Active tab highlighted with accent color
- Tab overflow: horizontal scroll with arrow buttons

### 1.4 — Create `TilingLayout.svelte` (recursive layout renderer)

The core layout engine.

**File:** `client/src/lib/components/workspace/TilingLayout.svelte`

Recursive component that renders `LayoutNode`:
- `type: "split"` → renders two children with a draggable divider between them
- `type: "widget"` → renders the widget component from the registry
- `type: "tabs"` → renders tab headers + the active child widget

**Sub-components:**
- `client/src/lib/components/workspace/SplitPane.svelte` — two children + resizable divider
- `client/src/lib/components/workspace/WidgetTabs.svelte` — tab group header + content

**Drag-and-drop zones:** When dragging a widget, each existing panel shows drop zones (left/right/top/bottom edges + center for tabbing). Dropping creates a new split node or adds to tab group.

### 1.5 — Rust filesystem commands

**File:** `client/src-tauri/src/fs_commands.rs`

New Tauri commands:
```rust
#[tauri::command]
async fn fs_read_dir(path: String) -> Result<Vec<FileEntry>, String>

#[tauri::command]
async fn fs_read_file(path: String) -> Result<Vec<u8>, String>

#[tauri::command]
async fn fs_write_file(path: String, content: Vec<u8>) -> Result<(), String>

#[tauri::command]
async fn fs_delete(path: String, recursive: bool) -> Result<(), String>

#[tauri::command]
async fn fs_rename(old_path: String, new_path: String) -> Result<(), String>

#[tauri::command]
async fn fs_stat(path: String) -> Result<FileStat, String>

// FileEntry: { name, path, is_dir, size, modified }
// FileStat: { size, modified, created, is_dir, is_file }
```

**Also update:** `client/src-tauri/src/lib.rs` to register the new commands in `invoke_handler`.
**Also update:** `client/src-tauri/Cargo.toml` if any new crates needed (likely just `serde_json` already present).

### 1.6 — Rust filesystem watcher

**File:** `client/src-tauri/src/fs_watcher.rs`

Uses the `notify` crate to watch directories for changes and emit events to the frontend.

```rust
#[tauri::command]
async fn fs_watch(path: String, app: tauri::AppHandle) -> Result<String, String>
// Returns a watch_id. Emits "fs-change" events: { watch_id, path, kind: "create"|"modify"|"delete" }

#[tauri::command]
async fn fs_unwatch(watch_id: String) -> Result<(), String>
```

**Update:** `client/src-tauri/Cargo.toml` — add `notify = "7"` dependency.

### 1.7 — `FileSystemProvider` interface + `LocalFileSystem`

**File:** `client/src/lib/filesystem/types.ts`
```typescript
interface FileEntry { name: string; path: string; isDir: boolean; size: number; modified: number }
interface FileStat { size: number; modified: number; created: number; isDir: boolean }
interface FileContent { data: Uint8Array; encoding?: string }
type FileChangeKind = "create" | "modify" | "delete"
interface FileChangeEvent { path: string; kind: FileChangeKind }
interface Disposable { dispose(): void }

interface FileSystemProvider {
  scheme: string  // "local", "remote", "cloud"
  readDir(path: string): Promise<FileEntry[]>
  readFile(path: string): Promise<FileContent>
  writeFile(path: string, content: string | Uint8Array): Promise<void>
  deleteFile(path: string): Promise<void>
  rename(oldPath: string, newPath: string): Promise<void>
  stat(path: string): Promise<FileStat>
  watch(path: string, callback: (event: FileChangeEvent) => void): Disposable
}
```

**File:** `client/src/lib/filesystem/local.ts`
- Implements `FileSystemProvider` using Tauri invoke commands from step 1.5/1.6

**File:** `client/src/lib/filesystem/index.ts`
- Exports `UnifiedFileSystem` that routes `local://`, `remote://`, `cloud://` paths to the right provider

### 1.8 — `FileExplorer.svelte` widget

**File:** `client/src/lib/components/workspace/FileExplorer.svelte`

- Tree view component with expand/collapse directories
- Icons for file types (folder, code, image, PDF, etc.)
- Root nodes: "Local" (from `LocalFileSystem`), "Workspace" (placeholder for phase 5), "Cloud" (placeholder)
- Single-click to select, double-click to open in editor
- Right-click context menu (using a `ContextMenu.svelte` component)
- Search/filter bar at the bottom
- File watcher integration — auto-refresh when files change

**Sub-component:** `client/src/lib/components/workspace/FileTreeNode.svelte` — recursive tree node

### 1.9 — Integrate into existing app layout

**Update:** `client/src/lib/components/AppShell.svelte`
- Add `TabBar` between TitleBar and main content
- Route tab content: chat tab → existing chat page, workspace tab → `TilingLayout`

**Update:** `client/src/routes/+page.svelte`
- Wrap in tab-aware layout — the current chat becomes the content of the default chat tab

**New route:** `client/src/routes/workspace/+page.svelte`
- Renders `TilingLayout` for the active workspace tab

### 1.10 — Mobile responsive foundation

**Update:** `client/src/lib/components/workspace/TilingLayout.svelte`
- `< 640px`: Render only the active leaf node, bottom tab bar to switch
- `640-1024px`: Render max two panes side-by-side
- `> 1024px`: Full tiling layout

**New component:** `client/src/lib/components/workspace/MobileZoneSwitcher.svelte`
- Bottom tab bar for mobile: Files, Editor, Chat, Widgets
- Swipe gesture support

---

## Phase 2: Editor & File Preview

### 2.1 — Monaco Editor widget

**File:** `client/src/lib/components/workspace/MonacoEditor.svelte`

- Wrap `monaco-editor` in a Svelte 5 component
- Props: `filePath`, `content`, `language`, `readOnly`
- Events: `onSave`, `onChange`, `onCursorChange`
- Auto-detect language from file extension
- Dark mode sync with app theme
- Minimap, line numbers, word wrap via settings

**File:** `client/src/lib/components/workspace/EditorTabs.svelte`
- Tab bar for open files (above Monaco)
- Dirty indicator (dot on unsaved tabs)
- Close tab, reorder tabs
- Tab overflow scroll

### 2.2 — File preview renderers

Each renderer is a Svelte component that takes `{ path, content }` props.

**Files:**
- `client/src/lib/components/workspace/previews/PdfPreview.svelte` — `pdfjs-dist`, page nav, zoom
- `client/src/lib/components/workspace/previews/ImagePreview.svelte` — `<img>` + pinch-zoom + pan
- `client/src/lib/components/workspace/previews/SpreadsheetPreview.svelte` — `xlsx` parser + table grid
- `client/src/lib/components/workspace/previews/MarkdownPreview.svelte` — `marked` rendered + raw toggle
- `client/src/lib/components/workspace/previews/JsonPreview.svelte` — collapsible tree + raw Monaco
- `client/src/lib/components/workspace/previews/HtmlPreview.svelte` — sandboxed `<iframe>`
- `client/src/lib/components/workspace/previews/MediaPreview.svelte` — `<video>` / `<audio>` player
- `client/src/lib/components/workspace/previews/HexPreview.svelte` — hex dump for binary files

**File:** `client/src/lib/components/workspace/FilePreview.svelte`
- Dispatcher component: reads file extension → selects the right preview renderer
- Falls back to Monaco for text, HexPreview for binary

### 2.3 — Diff visualization

**File:** `client/src/lib/components/workspace/DiffView.svelte`
- Uses `diff2html` to render side-by-side or inline diffs
- Props: `oldContent`, `newContent`, `filePath`
- Toggleable: side-by-side vs inline mode
- Used by Git widget and co-work file edit events

---

## Phase 3: Widget System

### 3.1 — Widget registry

**File:** `client/src/lib/widgets/registry.ts`

```typescript
class WidgetRegistry {
  private widgets = new Map<string, WidgetDefinition>()

  register(widget: WidgetDefinition): void
  unregister(id: string): void
  get(id: string): WidgetDefinition | undefined
  getAll(): WidgetDefinition[]
  getByCategory(cat: string): WidgetDefinition[]
}

// Singleton export
export const widgetRegistry = new WidgetRegistry()
```

**File:** `client/src/lib/widgets/types.ts`
- `WidgetDefinition` interface (id, name, icon, component, category, canPopOut, etc.)

**File:** `client/src/lib/widgets/index.ts`
- Registers all built-in widgets on import

### 3.2 — Built-in widgets

Each widget is a Svelte component that renders inside the tiling layout.

**Files:**
- `client/src/lib/widgets/builtin/ChatWidget.svelte` — Reuses existing `ChatPanel` component
- `client/src/lib/widgets/builtin/ActivityWidget.svelte` — Reuses existing activity log
- `client/src/lib/widgets/builtin/FileExplorerWidget.svelte` — Wraps `FileExplorer.svelte`
- `client/src/lib/widgets/builtin/EditorWidget.svelte` — Wraps `EditorTabs` + `MonacoEditor`
- `client/src/lib/widgets/builtin/PreviewWidget.svelte` — Wraps `FilePreview`

### 3.3 — `WidgetHeader.svelte`

**File:** `client/src/lib/components/workspace/WidgetHeader.svelte`

Universal header bar for every widget in the tiling layout:
- Widget name + icon
- Pop-out button (opens in separate window)
- Minimize/collapse toggle
- Close button (removes from layout)
- Draggable for repositioning

### 3.4 — Pop-out window support

**File:** `client/src-tauri/src/widget_windows.rs`

Rust commands for managing pop-out widget windows:
```rust
#[tauri::command]
async fn create_widget_window(widget_id: String, title: String, width: f64, height: f64, app: AppHandle) -> Result<(), String>

#[tauri::command]
async fn close_widget_window(widget_id: String, app: AppHandle) -> Result<(), String>
```

**New route:** `client/src/routes/widget/[id]/+page.svelte`
- Renders a single widget by ID in an isolated page (for the pop-out window)
- Connects to bridge for state sync

**Update:** `client/src/lib/tauri/bridge.ts`
- Add widget-specific bridge events: `pp:widget-state-sync`, `pp:widget-action`

### 3.5 — Widget store

**File:** `client/src/lib/stores/widget.svelte.ts`

```typescript
// State
- poppedOutWidgets: Set<string>  // widget IDs currently in separate windows
- widgetStates: Map<string, any>  // per-widget persisted state

// Methods
- popOut(widgetId): void
- popIn(widgetId): void
- getWidgetState(id): any
- setWidgetState(id, state): void
```

---

## Phase 4: Co-work & Terminal

### 4.1 — Terminal PTY backend (Rust)

**File:** `client/src-tauri/src/pty.rs`

Uses `portable-pty` crate (cross-platform PTY):
```rust
#[tauri::command]
async fn pty_spawn(shell: Option<String>, cwd: Option<String>, app: AppHandle) -> Result<String, String>
// Returns pty_id. Emits "pty-output" events with { pty_id, data: String }

#[tauri::command]
async fn pty_write(pty_id: String, data: String) -> Result<(), String>

#[tauri::command]
async fn pty_resize(pty_id: String, cols: u16, rows: u16) -> Result<(), String>

#[tauri::command]
async fn pty_kill(pty_id: String) -> Result<(), String>
```

**Update:** `client/src-tauri/Cargo.toml` — add `portable-pty = "0.8"`.

### 4.2 — Terminal widget

**File:** `client/src/lib/widgets/builtin/TerminalWidget.svelte`

- Renders `xterm.js` terminal
- On mount: calls `pty_spawn`, listens for `pty-output` events
- On keypress: sends to `pty_write`
- On resize: calls `pty_resize` with `@xterm/addon-fit`
- Supports multiple terminal instances (tabs within the widget)
- Agent output rendered with a distinct left-border color

### 4.3 — Agent file event streaming

**Update:** `client/src/lib/api/websocket.ts`
- Add handlers for new event types: `file_open`, `file_edit`, `file_create`, `file_delete`, `terminal_output`

**Update:** `client/src/lib/stores/workspace.svelte.ts`
- On `file_open`: auto-open the file in the editor widget
- On `file_edit`: update editor content, flash changed lines
- On `file_create`: refresh file explorer, optionally open
- On `file_delete`: close editor tab if open, refresh explorer
- On `terminal_output`: forward to terminal widget

**Backend (Python) changes needed:**
- **File:** `src/pocketpaw/bus/events.py` — Add `AgentFileEvent` types to `SystemEvent`
- **File:** `src/pocketpaw/agents/loop.py` — Emit file events when agent uses file tools
- **File:** `src/pocketpaw/dashboard/websocket.py` — Forward file events to connected clients

### 4.4 — Live cursor presence

**Update:** `client/src/lib/components/workspace/MonacoEditor.svelte`
- Add decoration API for remote cursors
- Listen for `cursor_position` WebSocket events from the agent
- Render agent cursor as a colored line with a name label
- User cursor in default color, agent cursor in accent color

### 4.5 — Command Palette

**File:** `client/src/lib/components/workspace/CommandPalette.svelte`

- Modal overlay triggered by `Cmd+K` / `Ctrl+K`
- Search input with fuzzy matching
- Categorized commands:
  - **Agent:** "Ask agent to...", "Run tests", "Explain this file", "Fix errors"
  - **Files:** "Open file", "Search in files", "New file", "Save all"
  - **Workspace:** "Save layout preset", "Load preset", "Toggle terminal", "Reset layout"
  - **Navigate:** "Go to line", "Go to file", "Switch tab"
- Recent commands section
- Keyboard navigation (arrow keys + enter)

### 4.6 — Destructive action toasts

**File:** `client/src/lib/components/workspace/UndoToast.svelte`

- Appears at bottom-center when agent performs destructive action (delete, overwrite)
- Shows action description + "Undo" button
- Auto-dismisses after 5 seconds
- Undo calls the inverse operation (restore from backup)

**File:** `client/src/lib/stores/undo.svelte.ts`
- Manages undo stack for destructive file operations
- Backs up file content before destructive action

---

## Phase 5: Remote Files, Cloud Storage & Polish

### 5.1 — Remote filesystem provider

**File:** `client/src/lib/filesystem/remote.ts`
- Implements `FileSystemProvider` using REST API calls
- Endpoints: `GET /api/v1/workspace/files`, `GET/PUT/DELETE /api/v1/workspace/files/{path}`
- WebSocket-based watch (listens for agent file events)

**Backend (Python):**
- **New file:** `src/pocketpaw/dashboard/routes/workspace.py`
  - `GET /api/v1/workspace/files` — list files in agent workspace
  - `GET /api/v1/workspace/files/{path}` — read file
  - `PUT /api/v1/workspace/files/{path}` — write file
  - `DELETE /api/v1/workspace/files/{path}` — delete file

### 5.2 — Cloud storage providers

**File:** `client/src/lib/filesystem/cloud/google-drive.ts`
- Implements `FileSystemProvider` for Google Drive
- Uses Google Drive REST API v3
- OAuth handled by existing auth system

**File:** `client/src/lib/filesystem/cloud/dropbox.ts`
- Implements `FileSystemProvider` for Dropbox
- Uses Dropbox HTTP API

**File:** `client/src/lib/filesystem/cloud/s3.ts`
- Implements `FileSystemProvider` for S3-compatible storage
- Configurable endpoint for R2, MinIO, etc.

**Update:** Settings page to configure cloud storage accounts.

### 5.3 — Layout presets

**Update:** `client/src/lib/stores/workspace.svelte.ts`
- Built-in presets: "Coding" (default), "Review" (editor + diff + git), "Research" (chat + file explorer), "Minimal" (just chat)
- Custom presets saved to localStorage
- Preset switcher in tab bar or command palette

### 5.4 — Mobile responsive polish

**Update:** All workspace components for responsive breakpoints
- `TilingLayout.svelte`: Responsive rendering (single/dual/full)
- `MobileZoneSwitcher.svelte`: Bottom nav + swipe gestures
- `TabBar.svelte`: Scrollable on mobile, compact mode
- `CommandPalette.svelte`: Full-screen on mobile
- Touch-friendly: larger hit targets, swipe to close panels

---

## Phase 6: Extensions & Git

### 6.1 — Git widget

**File:** `client/src/lib/widgets/builtin/GitWidget.svelte`

- Shows current branch, status (modified/staged/untracked files)
- Inline diff viewer for each changed file
- Stage/unstage, commit (with message input), push/pull buttons
- Branch switcher dropdown
- Uses Tauri commands that shell out to `git` CLI

**File:** `client/src-tauri/src/git_commands.rs`
```rust
#[tauri::command]
async fn git_status(cwd: String) -> Result<GitStatus, String>

#[tauri::command]
async fn git_diff(cwd: String, path: Option<String>) -> Result<String, String>

#[tauri::command]
async fn git_stage(cwd: String, paths: Vec<String>) -> Result<(), String>

#[tauri::command]
async fn git_commit(cwd: String, message: String) -> Result<(), String>

#[tauri::command]
async fn git_log(cwd: String, limit: u32) -> Result<Vec<GitLogEntry>, String>

#[tauri::command]
async fn git_branches(cwd: String) -> Result<Vec<String>, String>

#[tauri::command]
async fn git_checkout(cwd: String, branch: String) -> Result<(), String>
```

### 6.2 — MCP tool widget auto-registration

**Update:** `client/src/lib/widgets/registry.ts`
- When MCP servers connect and report tools with UI capabilities, auto-register a generic widget that renders tool output
- Listen for `mcp_connected` / `mcp_disconnected` events

**File:** `client/src/lib/widgets/mcp/McpToolWidget.svelte`
- Generic widget that can invoke an MCP tool and display its output
- Input form auto-generated from tool schema
- Output rendered as formatted JSON, text, or custom component

### 6.3 — Skill widget support

**Update:** `client/src/lib/widgets/registry.ts`
- Skills can declare a `widget` field in their manifest
- On skill install, register the widget component
- Widget component loaded dynamically via dynamic import

---

## Dependency Graph

```
Phase 1.1 (deps) ──┐
Phase 1.2 (store) ──┤
Phase 1.5 (rust fs)─┼──→ Phase 1.7 (fs provider) ──→ Phase 1.8 (explorer)
Phase 1.6 (watcher)─┘                                       │
Phase 1.3 (tab bar) ────────────────────────────────────────┤
Phase 1.4 (tiling) ─────────────────────────────────────────┼──→ Phase 1.9 (integrate)
                                                             │         │
Phase 2.1 (monaco) ──→ Phase 2.2 (previews) ──→ Phase 2.3 (diff)     │
                                                                       │
Phase 3.1 (registry) ──→ Phase 3.2 (builtins) ──→ Phase 3.3 (header)──┘
Phase 3.4 (pop-out) ──→ Phase 3.5 (widget store)

Phase 4.1 (pty rust) ──→ Phase 4.2 (terminal widget)
Phase 4.3 (file events) ──→ Phase 4.4 (cursors)
Phase 4.5 (cmd palette)
Phase 4.6 (undo toasts)

Phase 5.1 (remote fs)
Phase 5.2 (cloud)
Phase 5.3 (presets)
Phase 5.4 (mobile)

Phase 6.1 (git widget)
Phase 6.2 (mcp widgets)
Phase 6.3 (skill widgets)
```

## File Count Summary

| Category | New Files | Modified Files |
|----------|-----------|----------------|
| Rust (src-tauri) | 5 | 2 (lib.rs, Cargo.toml) |
| Stores | 3 | 1 (index.ts) |
| Workspace components | 12 | 2 (AppShell, +page) |
| Preview components | 8 | 0 |
| Widget system | 10 | 1 (bridge.ts) |
| Filesystem | 6 | 0 |
| Backend (Python) | 3 | 3 |
| Routes | 2 | 1 |
| **Total** | **~49 new** | **~10 modified** |
