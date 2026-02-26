# Multi-Mode Workspace Desktop Client Design

**Date:** 2026-02-26
**Status:** Approved
**Scope:** PocketPaw desktop client (`client/`) — Tauri 2.0 + SvelteKit 2 + Svelte 5

## Overview

Transform the PocketPaw desktop client from a chat-first app into a full workspace IDE with multiple modes, a tiling layout engine, pluggable widgets, real-time agent collaboration, unified file management, and responsive mobile support.

## Architecture Decision

**Hybrid SPA + Pop-out:** Core workspace is a SvelteKit SPA within the existing Tauri app. Any widget panel can be popped out into a separate Tauri webview window. State sync between main window and popped-out windows uses the existing `bridge.ts` cross-window event system.

## 1. Workspace Tab System

Horizontal tab bar below the title bar. Each tab is an independent workspace.

```
┌─────────────────────────────────────────────────────────┐
│  [←][→]  PocketPaw           ─ □ ✕                      │  Title bar
├─────────────────────────────────────────────────────────┤
│  [+ New] [💬 Chat] [📁 Project X] [📁 Debug logs]  ... │  Tab bar
├─────────────────────────────────────────────────────────┤
│              (Tab content here)                          │
└─────────────────────────────────────────────────────────┘
```

### Tab Types

- **Chat tab** — Full-width chat interface (existing UI, enhanced). Default tab on launch.
- **Workspace tab** — Tiling layout with file explorer, editor/preview, and draggable widgets.

### Tab State (`workspaceStore`)

```typescript
interface Tab {
  id: string
  type: "chat" | "workspace"
  title: string
  sessionId: string          // Links to backend session
  layoutTree?: LayoutNode    // Workspace layout (null for chat tabs)
  openFiles?: string[]       // Open file tabs in editor
}
```

### Tab Behavior

- New tab defaults to Chat mode. "Switch to Workspace" button or automatic transition when agent starts file operations.
- Tabs persist across sessions (saved to local storage).
- Close with middle-click or X button. Last tab creates a new empty one.
- Drag to reorder.

## 2. Workspace Mode Layout

Tiling layout engine where widgets can be placed anywhere.

### Layout Tree

```typescript
interface LayoutNode {
  type: "split" | "widget" | "tabs"
  direction?: "horizontal" | "vertical"  // for "split"
  ratio?: number                          // split ratio (0-1)
  children?: LayoutNode[]                 // for "split" and "tabs"
  widgetId?: string                       // for "widget" leaf nodes
}
```

### Default Layout Preset ("Coding")

```
┌──────────┬────────────────────────┬──────────┐
│          │                        │  Chat    │
│  Files   │     Editor (tabs)      │──────────│
│          │                        │ Terminal  │
│          ├────────────────────────│──────────│
│          │     Diff / Preview     │ Activity  │
└──────────┴────────────────────────┴──────────┘
```

### Layout Features

- Drag a widget to create a new split (left, right, top, bottom of any existing panel)
- Drop a widget onto another to create a tabbed group
- Resize any split boundary by dragging dividers
- Double-click divider to reset to default ratio
- Save/load layout presets (e.g., "Coding", "Review", "Research")

## 3. File System Provider

Unified abstraction for local, remote, and cloud files.

```typescript
interface FileSystemProvider {
  readDir(path: string): Promise<FileEntry[]>
  readFile(path: string): Promise<FileContent>
  writeFile(path: string, content: string | Uint8Array): Promise<void>
  deleteFile(path: string): Promise<void>
  rename(oldPath: string, newPath: string): Promise<void>
  stat(path: string): Promise<FileStat>
  watch(path: string, callback: (event: FileChangeEvent) => void): Disposable
}
```

### Implementations

1. **`LocalFileSystem`** — Tauri Rust commands (`fs_read_dir`, `fs_read_file`, `fs_write_file`, `fs_delete`, `fs_rename`, `fs_stat`, `fs_watch`). Uses Rust `notify` crate for filesystem events. Scoped to user-approved directories via Tauri's `fs` plugin.

2. **`RemoteFileSystem`** — REST API calls to PocketPaw backend. Agent workspace files in `~/.pocketpaw/workspace/`.

3. **`CloudFileSystem`** — Cloud storage APIs:
   - Google Drive (OAuth)
   - Dropbox (API)
   - S3/R2 (AWS SDK compatible)
   - Configured in Settings. Each provider implements `FileSystemProvider`.

### File Explorer

- Tree view with root nodes: "Local", "Workspace" (remote), "Cloud"
- Path prefixes: `local://`, `remote://`, `cloud://<provider>/`
- Right-click context menu: Open, Rename, Delete, Copy Path, Open in System
- Search bar for quick file finding
- Collapsible to icon-only (~48px)

## 4. Widget System

### Widget Interface

```typescript
interface Widget {
  id: string
  name: string
  icon: string                        // Lucide icon name
  component: typeof SvelteComponent   // Svelte component to render
  defaultWidth?: number
  defaultHeight?: number
  canPopOut?: boolean                  // Can be popped into separate window
  category: "builtin" | "mcp" | "skill" | "extension"
  onActivate?(): void
  onDeactivate?(): void
  onPopOut?(): void
}

class WidgetRegistry {
  register(widget: Widget): void
  unregister(id: string): void
  getAll(): Widget[]
  getByCategory(cat: string): Widget[]
}
```

### Built-in Widgets

| Widget | Description |
|--------|-------------|
| Chat | Chat interface (messages + input) |
| Terminal | PTY terminal via Tauri + xterm.js |
| Activity | Real-time agent activity log |
| Git | Branch, status, diff, commit controls |
| File Preview | Quick preview of selected file |
| File Explorer | Directory tree browser |
| Editor | Monaco-based code editor with file tabs |
| Command Palette | Searchable agent actions |

### External Widget Registration

- **MCP tools:** Auto-register widgets when a connected MCP server provides UI-capable tools.
- **Skills:** Declare `sidebar_widget` in skill manifest. Component loads into layout.

### Pop-out Behavior

- Click "pop out" icon on widget header → new Tauri webview window rendering just that widget
- State synced via bridge system (same as existing side panel sync)
- Closing popped-out window returns widget to the main layout

## 5. Co-work (Real-time Agent Collaboration)

### Agent Operations Streaming

Agent file operations stream to the workspace in real-time via WebSocket:

```typescript
type AgentFileEvent =
  | { type: "file_open", path: string }
  | { type: "file_edit", path: string, diff: Diff }
  | { type: "file_create", path: string }
  | { type: "file_delete", path: string }
  | { type: "terminal_output", content: string }
```

### Simplified Approval Flow

- Agent works freely — no approval pop-ups interrupting flow
- All changes stream to the workspace in real-time (files open, diffs flash, terminal scrolls)
- User reviews changes via Git widget (status, diff, commit)
- Toast notification + 5-second "Undo" button for destructive actions only (delete, overwrite)

### Live Cursors

- Agent editing cursor shown in a distinct color in the editor
- User cursor in a different color
- Similar to Google Docs collaboration cursors

### Command Palette (`Cmd+K` / `Ctrl+K`)

- Agent actions: "Ask agent to...", "Run tests", "Explain this file", "Fix errors", "Refactor selection"
- File actions: "Open file", "Search in files", "New file"
- Workspace actions: "Save layout", "Switch preset", "Toggle terminal"

### Terminal Integration

- Full PTY terminal via `xterm.js` + Tauri PTY commands
- Agent's shell commands appear in real-time
- User can type in the same terminal
- Agent output visually distinct (colored differently)

## 6. File Preview System

| File Type | Renderer | Library | Editable |
|-----------|----------|---------|----------|
| Code (.py, .ts, .rs, etc.) | Monaco Editor | `monaco-editor` | Yes |
| PDF | Page viewer with nav | `pdf.js` | No |
| Excel/CSV | Table grid | `SheetJS` + custom | Read-only (v1) |
| Images (.png, .jpg, .svg) | Zoomable viewer | Native `<img>` + pan/zoom | No |
| Markdown (.md) | Split: raw + rendered | `marked` or `mdsvex` | Yes (raw side) |
| JSON | Tree viewer + raw | Custom tree + Monaco | Yes |
| HTML | Rendered preview | `<iframe>` sandbox | Source editable |
| Video/Audio | Native player | `<video>` / `<audio>` | No |
| Unknown/Binary | Hex viewer | Custom component | No |

### Monaco Editor Config

- Syntax highlighting for 50+ languages
- Agent edit highlighting (changed lines glow briefly)
- Minimap, line numbers, word wrap toggle
- Diff view mode (side-by-side or inline)

## 7. Mobile Adaptation

### Phone (< 640px)

```
┌─────────────────────┐
│                     │
│   ACTIVE ZONE       │
│   (full screen)     │
│   One zone visible  │
│   at a time         │
│                     │
├─────────────────────┤
│  [📁] [📝] [💬] [⚙] │  Bottom tab bar
└─────────────────────┘
     [🔍]               Floating command button
```

- Bottom tab bar switches between zones: Files, Editor, Chat, Widgets
- One zone visible at a time (full screen)
- Swipe left/right to switch zones
- File tap → switches to Editor zone
- Widgets stack vertically in scrollable list
- No pop-out windows

### Tablet (640-1024px)

- Two-zone split side-by-side (like iPad multitasking)
- File explorer + Editor, or Editor + Chat
- Widgets as slide-over panel from right edge
- Drag-and-drop between zones

### Desktop (> 1024px)

- Full tiling workspace layout
- All features available

### Shared State

`workspaceStore` holds the same layout tree regardless of platform. Responsive rendering:
- Mobile: shows one leaf node at a time
- Tablet: shows two leaf nodes side-by-side
- Desktop: renders the full tree

## 8. New Components Needed

### Rust (src-tauri)

- `fs_commands.rs` — Local filesystem operations (read_dir, read_file, write_file, delete, rename, stat)
- `fs_watcher.rs` — Filesystem watch via `notify` crate, emit events to frontend
- `pty.rs` — PTY terminal management (spawn shell, read/write, resize)
- `widget_windows.rs` — Pop-out widget window creation and management

### Frontend (SvelteKit)

**Stores:**
- `workspaceStore` — Tab management, layout tree, active tab, presets
- `fileSystemStore` — File tree state, open files, providers
- `widgetStore` — Widget registry, layout positions

**Components:**
- `TabBar.svelte` — Workspace tab bar
- `TilingLayout.svelte` — Recursive tiling layout renderer
- `FileExplorer.svelte` — Tree view with context menus
- `EditorTabs.svelte` — Monaco editor with file tabs
- `FilePreview.svelte` — Type-dispatching preview renderer
- `TerminalWidget.svelte` — xterm.js terminal
- `GitWidget.svelte` — Git status, diff, commit
- `CommandPalette.svelte` — Searchable action list
- `WidgetHeader.svelte` — Widget chrome (title, pop-out, close)

**Libraries to add:**
- `monaco-editor` — Code editing
- `xterm.js` + `@xterm/addon-fit` — Terminal
- `pdfjs-dist` — PDF rendering
- `xlsx` (SheetJS) — Excel/CSV
- `diff` or `diff2html` — Diff visualization

### Backend (Python)

- New WebSocket events for agent file operations (`file_open`, `file_edit`, `file_create`, `file_delete`, `terminal_output`)
- File API endpoints for remote workspace (`/api/v1/workspace/files/*`)
- PTY management for terminal sessions

## 9. Implementation Phases

### Phase 1: Foundation
- Tab system + workspaceStore
- Basic tiling layout engine
- File explorer (local only) with Tauri fs commands

### Phase 2: Editor & Preview
- Monaco editor integration
- File preview system (PDF, images, markdown, JSON)
- Editor file tabs

### Phase 3: Widgets
- Widget registry + built-in widgets
- Chat widget, Activity widget
- Pop-out window support

### Phase 4: Co-work
- Agent file event streaming
- Live cursor presence
- Terminal widget (PTY)
- Command palette

### Phase 5: Cloud & Polish
- Remote filesystem provider
- Cloud storage providers (Google Drive, Dropbox, S3)
- Layout presets
- Mobile responsive adaptation

### Phase 6: Extensions
- MCP tool widget auto-registration
- Skill widget support
- Git widget
