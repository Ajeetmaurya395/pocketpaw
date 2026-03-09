# Mode-Based Left Sidebar Design

**Date:** 2026-03-09
**Status:** Approved

## Problem

The left sidebar currently shows the same content (chat sessions + footer tools) regardless of the active mode. In Files mode, showing chat sessions is redundant since the explorer already has a collapsible chat panel on the right (`ChatSidebar.svelte`). The sidebar should adapt to what's contextually useful.

## Design

### Mode Detection

Two sidebar modes determined by the current route:

| Route | Sidebar Mode |
|---|---|
| `/` (Files/Explorer) | **Files sidebar** |
| `/chat` | **Chat sidebar** |
| `/settings`, `/memory`, `/identity`, `/mcp`, `/health`, `/metrics`, `/activity`, `/explore` | **Chat sidebar** |

Detection logic lives in `AppSidebar.svelte` using SvelteKit's `$page.url.pathname`.

### Chat Sidebar (default)

Shown on `/chat` and all tool pages. Enhances the existing sidebar with:

#### 1. Quick Filter Chips (below search)

Horizontal scrollable row of small filter chips below the session search input:

- **Today** - sessions from today only
- **With files** - sessions that have file attachments
- **Starred** - only pinned/starred sessions

Chips are toggleable. Active chip gets accent styling. Only one active at a time (tap again to deselect).

#### 2. Pinned/Starred Sessions Section

- Star icon button on each session item (on hover or always visible on touch)
- Starred sessions appear in a "Pinned" group at the top, above date-grouped sessions
- Pinned section is collapsible
- Star state stored per session (in session metadata, persisted via API)

#### 3. Session Model Badge

- Small subtle badge/icon on each session item showing which agent backend was used
- Positioned to the right of the session title or below as secondary text
- Shows abbreviated backend name or icon (e.g. Claude icon, GPT icon, Gemini icon)
- Only shown if the session has model metadata available

#### 4. Footer Tools Grid

Unchanged from current implementation. 3-column grid with Activity, Memory, Identity, Skills, Health, Metrics, MCP, Settings.

### Files Sidebar

Shown on `/` (Files/Explorer route). Replaces the session list entirely.

#### 1. Quick Access Section

Collapsible section with icon + label rows:

- **Recent Files** - last 10 opened files (from explorer store or API)
- **Favorites** - user-pinned files/folders (star action in explorer)
- **Downloads** - shortcut to system Downloads folder

Each item navigates the explorer to that location on click.

#### 2. Locations Section

Collapsible section with system directory shortcuts:

- **Home** - user home directory
- **Desktop** - system desktop folder
- **Documents** - system documents folder

Each item navigates the explorer to that path. Paths resolved via Tauri filesystem API or backend endpoint.

#### 3. Cloud Storage Section

Collapsible section with disabled integration buttons:

- **Google Drive** - disabled, "Coming soon" tooltip
- **Dropbox** - disabled, "Coming soon" tooltip
- **OneDrive** - disabled, "Coming soon" tooltip

Each shows the service icon + name with muted/disabled styling. Hover shows "Coming soon" tooltip. These are placeholders for future integrations.

#### 4. Footer Tools Grid

Same footer as Chat sidebar. Shared component, always present at the bottom.

## Component Changes

### Modified Components

- **`AppSidebar.svelte`** - Add route-based mode detection. Conditionally render `SidebarSessions` (chat mode) or new `SidebarExplorer` (files mode).
- **`SidebarSessions.svelte`** - Add filter chips row below search. Add pinned sessions section. Add model badge to session items.
- **`SidebarFooter.svelte`** - No changes (shared across both modes).

### New Components

- **`SidebarExplorer.svelte`** - Files mode sidebar content. Contains Quick Access, Locations, and Cloud Storage sections.
- **`SidebarSection.svelte`** (optional) - Reusable collapsible section wrapper with header + chevron toggle.

### Store Changes

- **`sessionStore`** - Add `pinnedSessionIds` state. Add `togglePin(sessionId)` method. Expose `pinnedSessions` derived state.
- **`explorerStore`** - Add `recentFiles` state (last 10 opened). Add `favoriteFiles` state (user-pinned paths). Add methods: `addToRecent(path)`, `toggleFavorite(path)`.

## Data Flow

```
$page.url.pathname
    |
    v
AppSidebar (mode detection)
    |
    +-- "/": SidebarExplorer
    |         +-- Quick Access (explorerStore.recentFiles, explorerStore.favoriteFiles)
    |         +-- Locations (system paths via Tauri/backend)
    |         +-- Cloud Storage (static disabled items)
    |         +-- SidebarFooter
    |
    +-- else: SidebarSessions (enhanced)
              +-- Search
              +-- Filter Chips
              +-- Pinned Sessions (sessionStore.pinnedSessions)
              +-- Date-Grouped Sessions
              +-- SidebarFooter
```

## Styling

- Both sidebar modes share the same width, background, and border styling
- Filter chips: small rounded pills, muted bg, accent on active
- Pinned section: subtle separator or different bg tint
- Model badge: 12-14px text, muted color, right-aligned or second line
- Files sidebar sections: collapsible with animated chevron
- Cloud storage items: opacity-50, cursor-not-allowed, "Coming soon" tooltip
- All styling follows existing Tailwind + shadcn-svelte patterns
