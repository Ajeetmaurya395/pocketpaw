# Mode-Based Sidebar Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Make the left sidebar adapt its content based on the active route: Files sidebar on `/`, Chat sidebar everywhere else.

**Architecture:** Route-based mode detection in `AppSidebar.svelte` using `$page.url.pathname`. Two sidebar bodies: existing `SidebarSessions` (enhanced with filter chips, pinned sessions, model badges) and new `SidebarExplorer` (quick access, locations, cloud storage placeholders). Shared `SidebarFooter` at the bottom of both.

**Tech Stack:** Svelte 5 runes, Tailwind CSS 4, Lucide icons, shadcn-svelte Tooltip component, existing explorerStore/sessionStore

---

### Task 1: Create `SidebarSection.svelte` (Reusable Collapsible Section)

**Files:**
- Create: `client/src/lib/components/SidebarSection.svelte`

**Step 1: Create the collapsible section component**

```svelte
<script lang="ts">
  import { ChevronRight } from "@lucide/svelte";
  import type { Snippet } from "svelte";

  let {
    title,
    defaultOpen = true,
    children,
  }: {
    title: string;
    defaultOpen?: boolean;
    children: Snippet;
  } = $props();

  let open = $state(defaultOpen);
</script>

<div class="mb-1">
  <button
    type="button"
    onclick={() => (open = !open)}
    class="flex w-full items-center gap-1.5 px-3 py-1.5 text-[10px] font-medium uppercase tracking-wider text-muted-foreground/60 transition-colors hover:text-muted-foreground"
  >
    <ChevronRight
      class="h-3 w-3 shrink-0 transition-transform duration-150"
      style={open ? "transform: rotate(90deg)" : ""}
      strokeWidth={2}
    />
    {title}
  </button>
  {#if open}
    <div class="px-2 py-0.5">
      {@render children()}
    </div>
  {/if}
</div>
```

**Step 2: Verify it builds**

Run: `cd client && bun run check`

**Step 3: Commit**

```bash
git add client/src/lib/components/SidebarSection.svelte
git commit -m "feat(sidebar): add reusable collapsible SidebarSection component"
```

---

### Task 2: Create `SidebarExplorer.svelte` (Files Mode Sidebar)

**Files:**
- Create: `client/src/lib/components/SidebarExplorer.svelte`

**Dependencies:** Task 1 (SidebarSection)

The explorer store already has `defaultDirs` (home, documents, downloads, desktop) and `pinnedFolders`. The backend already has a recent files API (`client.getRecentFiles()`). We reuse these.

**Step 1: Create the files sidebar component**

```svelte
<script lang="ts">
  import {
    Clock,
    Star,
    Download,
    Home,
    Monitor,
    FileText,
    Cloud,
    Droplet,
    HardDrive,
  } from "@lucide/svelte";
  import type { RecentFileEntry } from "$lib/api/types";
  import SidebarSection from "./SidebarSection.svelte";
  import { explorerStore, connectionStore, platformStore } from "$lib/stores";
  import * as Tooltip from "$lib/components/ui/tooltip";

  let recentFiles = $state<RecentFileEntry[]>([]);

  let defaultDirs = $derived(explorerStore.defaultDirs);
  let pinnedFolders = $derived(explorerStore.pinnedFolders);

  // Load recent files on mount
  $effect(() => {
    loadRecentFiles();
  });

  async function loadRecentFiles() {
    try {
      const client = connectionStore.getClient();
      recentFiles = await client.getRecentFiles(10);
    } catch {
      // Backend may not support this yet
    }
  }

  function navigateTo(path: string) {
    explorerStore.navigateTo(path);
  }

  let itemClass = $derived(
    platformStore.isTouch
      ? "flex w-full items-center gap-2.5 rounded-md px-2.5 py-2 text-[13px] text-muted-foreground transition-colors active:bg-accent active:text-foreground"
      : "flex w-full items-center gap-2 rounded-md px-2 py-1.5 text-[12px] text-muted-foreground transition-colors hover:bg-accent hover:text-foreground"
  );

  const cloudServices = [
    { label: "Google Drive", icon: Cloud },
    { label: "Dropbox", icon: Droplet },
    { label: "OneDrive", icon: HardDrive },
  ] as const;
</script>

<div class="flex min-h-0 flex-1 flex-col overflow-y-auto">
  <!-- Quick Access -->
  <SidebarSection title="Quick Access">
    {#if recentFiles.length > 0}
      <button type="button" class={itemClass} onclick={() => {}}>
        <Clock class="h-3.5 w-3.5 shrink-0" strokeWidth={1.75} />
        <span class="truncate">Recent Files</span>
        <span class="ml-auto text-[10px] text-muted-foreground/50">{recentFiles.length}</span>
      </button>
    {/if}

    {#if pinnedFolders.length > 0}
      {#each pinnedFolders as folder (folder.path)}
        <button type="button" class={itemClass} onclick={() => navigateTo(folder.path)}>
          <Star class="h-3.5 w-3.5 shrink-0 text-amber-500/70" strokeWidth={1.75} />
          <span class="truncate">{folder.name}</span>
        </button>
      {/each}
    {:else}
      <button type="button" class={itemClass} onclick={() => {}}>
        <Star class="h-3.5 w-3.5 shrink-0" strokeWidth={1.75} />
        <span class="truncate">Favorites</span>
        <span class="ml-auto text-[10px] text-muted-foreground/40">None</span>
      </button>
    {/if}

    {#if defaultDirs?.downloads}
      <button type="button" class={itemClass} onclick={() => navigateTo(defaultDirs!.downloads)}>
        <Download class="h-3.5 w-3.5 shrink-0" strokeWidth={1.75} />
        <span class="truncate">Downloads</span>
      </button>
    {/if}
  </SidebarSection>

  <!-- Locations -->
  {#if defaultDirs}
    <SidebarSection title="Locations">
      <button type="button" class={itemClass} onclick={() => navigateTo(defaultDirs!.home)}>
        <Home class="h-3.5 w-3.5 shrink-0" strokeWidth={1.75} />
        <span class="truncate">Home</span>
      </button>
      <button type="button" class={itemClass} onclick={() => navigateTo(defaultDirs!.desktop)}>
        <Monitor class="h-3.5 w-3.5 shrink-0" strokeWidth={1.75} />
        <span class="truncate">Desktop</span>
      </button>
      <button type="button" class={itemClass} onclick={() => navigateTo(defaultDirs!.documents)}>
        <FileText class="h-3.5 w-3.5 shrink-0" strokeWidth={1.75} />
        <span class="truncate">Documents</span>
      </button>
    </SidebarSection>
  {/if}

  <!-- Cloud Storage (disabled/coming soon) -->
  <SidebarSection title="Cloud Storage" defaultOpen={false}>
    {#each cloudServices as service}
      {@const Icon = service.icon}
      <Tooltip.Root>
        <Tooltip.Trigger>
          <span
            class="flex w-full cursor-not-allowed items-center gap-2 rounded-md px-2 py-1.5 text-[12px] text-muted-foreground/40"
          >
            <Icon class="h-3.5 w-3.5 shrink-0" strokeWidth={1.75} />
            <span class="truncate">{service.label}</span>
          </span>
        </Tooltip.Trigger>
        <Tooltip.Content>Coming soon</Tooltip.Content>
      </Tooltip.Root>
    {/each}
  </SidebarSection>
</div>
```

**Step 2: Verify it builds**

Run: `cd client && bun run check`

**Step 3: Commit**

```bash
git add client/src/lib/components/SidebarExplorer.svelte
git commit -m "feat(sidebar): add SidebarExplorer with quick access, locations, cloud storage"
```

---

### Task 3: Add Route-Based Mode Switching to `AppSidebar.svelte`

**Files:**
- Modify: `client/src/lib/components/AppSidebar.svelte`

**Dependencies:** Task 2 (SidebarExplorer)

**Step 1: Add route detection and conditional rendering**

Import `page` from `$app/state` and `SidebarExplorer`. Derive `isFilesMode` from pathname. Conditionally render `SidebarSessions` vs `SidebarExplorer`.

The updated `AppSidebar.svelte`:

```svelte
<script lang="ts">
  import { X } from "@lucide/svelte";
  import { page } from "$app/state";
  import SidebarHeader from "./SidebarHeader.svelte";
  import SidebarSessions from "./SidebarSessions.svelte";
  import SidebarExplorer from "./SidebarExplorer.svelte";
  import SidebarFooter from "./SidebarFooter.svelte";

  let {
    isDrawer = false,
    onClose,
  }: {
    isDrawer?: boolean;
    onClose?: () => void;
  } = $props();

  let isFilesMode = $derived(page.url.pathname === "/");

  let sidebarClass = $derived(
    isDrawer
      ? "flex h-full w-full flex-col overflow-hidden bg-background border-r border-border"
      : "flex h-full w-full flex-col overflow-hidden"
  );
</script>

<aside class={sidebarClass} data-no-select>
  {#if isDrawer && onClose}
    <div class="flex items-center justify-between px-3 pt-2">
      <div class="flex items-center gap-2">
        <span class="text-lg">🐾</span>
        <span class="text-[13px] font-semibold text-foreground">PocketPaw</span>
      </div>
      <button
        onclick={onClose}
        class="flex h-8 w-8 items-center justify-center rounded-md text-muted-foreground transition-colors active:bg-foreground/10"
      >
        <X class="h-4 w-4" />
      </button>
    </div>
  {/if}
  {#if !isDrawer}
    <SidebarHeader />
  {:else}
    <SidebarHeader hideLogoRow />
  {/if}

  {#if isFilesMode}
    <SidebarExplorer />
  {:else}
    <SidebarSessions />
  {/if}

  <SidebarFooter />
</aside>
```

**Step 2: Verify it builds**

Run: `cd client && bun run check`

**Step 3: Commit**

```bash
git add client/src/lib/components/AppSidebar.svelte
git commit -m "feat(sidebar): route-based mode switching between chat and files sidebar"
```

---

### Task 4: Add Pinned Sessions to `sessionStore`

**Files:**
- Modify: `client/src/lib/stores/sessions.svelte.ts`

**Step 1: Add pinned session state and methods**

Add these properties and methods to `SessionStore`:

```typescript
// After line 3 (imports)
const PINNED_KEY = "pocketpaw_pinned_sessions";

// Inside the class, after isLoadingHistory:
pinnedSessionIds = $state<Set<string>>(new Set());

// Derived: pinned sessions in order
pinnedSessions = $derived(
  this.sessions.filter((s) => this.pinnedSessionIds.has(s.id))
);

// In the constructor or as a field initializer, load from localStorage:
// (add to the class body)
constructor() {
  try {
    const raw = localStorage.getItem(PINNED_KEY);
    if (raw) {
      this.pinnedSessionIds = new Set(JSON.parse(raw));
    }
  } catch {
    // Ignore
  }
}

// Methods:
togglePin(sessionId: string): void {
  const next = new Set(this.pinnedSessionIds);
  if (next.has(sessionId)) {
    next.delete(sessionId);
  } else {
    next.add(sessionId);
  }
  this.pinnedSessionIds = next;
  try {
    localStorage.setItem(PINNED_KEY, JSON.stringify([...next]));
  } catch {
    // localStorage unavailable
  }
}

isSessionPinned(sessionId: string): boolean {
  return this.pinnedSessionIds.has(sessionId);
}
```

**Step 2: Verify it builds**

Run: `cd client && bun run check`

**Step 3: Commit**

```bash
git add client/src/lib/stores/sessions.svelte.ts
git commit -m "feat(sessions): add pinned sessions state with localStorage persistence"
```

---

### Task 5: Add Filter Chips to `SidebarSessions.svelte`

**Files:**
- Modify: `client/src/lib/components/SidebarSessions.svelte`

**Dependencies:** Task 4 (pinned sessions in store)

**Step 1: Add filter chip state and UI**

Add after the `SessionSearch` component in the template, a row of filter chips. Add filter state and logic to the script.

Add to script section:

```typescript
type FilterType = "today" | "with_files" | "starred" | null;
let activeFilter = $state<FilterType>(null);

function toggleFilter(filter: FilterType) {
  activeFilter = activeFilter === filter ? null : filter;
}
```

Modify the `groupedSessions` derived to respect the active filter. When `activeFilter` is set, pre-filter `sessions` before grouping:

```typescript
let filteredSessionList = $derived.by(() => {
  if (!activeFilter) return sessions;
  const now = new Date();
  const today = new Date(now.getFullYear(), now.getMonth(), now.getDate());

  switch (activeFilter) {
    case "today":
      return sessions.filter((s) => new Date(s.last_activity) >= today);
    case "starred":
      return sessions.filter((s) => sessionStore.isSessionPinned(s.id));
    case "with_files":
      // Placeholder: show all for now (needs metadata from backend)
      return sessions;
    default:
      return sessions;
  }
});
```

Then update `groupedSessions` to use `filteredSessionList` instead of `sessions`.

Add the filter chips UI after `<SessionSearch>`:

```svelte
<!-- Filter chips -->
<div class="flex gap-1.5 px-3 pb-1.5">
  {#each [
    { key: "today", label: "Today" },
    { key: "starred", label: "Starred" },
    { key: "with_files", label: "With files" },
  ] as chip (chip.key)}
    <button
      type="button"
      onclick={() => toggleFilter(chip.key as FilterType)}
      class={activeFilter === chip.key
        ? "rounded-full bg-paw-accent-subtle px-2.5 py-0.5 text-[10px] font-medium text-foreground transition-colors"
        : "rounded-full bg-muted/60 px-2.5 py-0.5 text-[10px] text-muted-foreground transition-colors hover:bg-muted"}
    >
      {chip.label}
    </button>
  {/each}
</div>
```

Also add a "Pinned" section above the date-grouped sessions (when not searching and no filter active):

```svelte
{#if !showSearchResults && !activeFilter}
  {@const pinned = sessionStore.pinnedSessions}
  {#if pinned.length > 0}
    <p class="mt-1 px-2 pb-0.5 text-[10px] font-medium uppercase tracking-wider text-muted-foreground/50">
      Pinned
    </p>
    {#each pinned as session (session.id)}
      <SessionItem {session} isActive={session.id === activeId} />
    {/each}
  {/if}
{/if}
```

**Step 2: Verify it builds**

Run: `cd client && bun run check`

**Step 3: Commit**

```bash
git add client/src/lib/components/SidebarSessions.svelte
git commit -m "feat(sidebar): add filter chips and pinned sessions section"
```

---

### Task 6: Add Star/Pin Button and Model Badge to `SessionItem.svelte`

**Files:**
- Modify: `client/src/lib/components/SessionItem.svelte`

**Dependencies:** Task 4 (pinned sessions in store)

**Step 1: Add star button and model badge**

Import `Star` icon and `sessionStore` (already imported). Add pin toggle button. Add model badge display.

Add to the session button content, after the title span:

```svelte
<!-- Star/pin button -->
<button
  type="button"
  onclick={(e: MouseEvent) => {
    e.stopPropagation();
    sessionStore.togglePin(session.id);
  }}
  class={[
    "shrink-0 rounded-sm p-0.5 transition-colors",
    sessionStore.isSessionPinned(session.id)
      ? "text-amber-500"
      : platformStore.isTouch
        ? "text-transparent active:text-muted-foreground"
        : "text-transparent group-hover:text-muted-foreground/40 hover:!text-amber-500",
  ].join(" ")}
  title={sessionStore.isSessionPinned(session.id) ? "Unpin" : "Pin"}
>
  <Star
    class="h-3 w-3"
    strokeWidth={1.75}
    fill={sessionStore.isSessionPinned(session.id) ? "currentColor" : "none"}
  />
</button>
```

For the model badge, add after the star button or below the title if metadata is available:

```svelte
{#if session.channel && session.channel !== "websocket"}
  <span class="shrink-0 text-[9px] text-muted-foreground/50">{session.channel}</span>
{/if}
```

Rearrange the button content to accommodate the star icon without breaking the layout. The star replaces the relative time on hover (the relative time already hides on `group-hover`).

**Step 2: Verify it builds**

Run: `cd client && bun run check`

**Step 3: Commit**

```bash
git add client/src/lib/components/SessionItem.svelte
git commit -m "feat(sidebar): add star/pin button and channel badge to session items"
```

---

### Task 7: Update `SidebarHeader` for Files Mode

**Files:**
- Modify: `client/src/lib/components/SidebarHeader.svelte`

**Dependencies:** Task 3 (route detection)

The "New Chat" button doesn't make sense in Files mode. In files mode, show the logo but no "New Chat" button (or change it to something contextual).

**Step 1: Make header mode-aware**

Add route detection to conditionally show "New Chat" only in chat mode:

```svelte
<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import { Plus } from "@lucide/svelte";
  import { goto } from "$app/navigation";
  import { page } from "$app/state";
  import { sessionStore, platformStore } from "$lib/stores";

  let { hideLogoRow = false }: { hideLogoRow?: boolean } = $props();

  let isFilesMode = $derived(page.url.pathname === "/");

  function newChat() {
    sessionStore.createNewSession();
    if (page.url.pathname !== "/chat") {
      goto("/chat");
    }
  }

  let btnClass = $derived(
    platformStore.isTouch
      ? "w-full justify-start gap-2 text-[13px] touch-target"
      : "w-full justify-start gap-2 text-[12px]"
  );
</script>

<div class="flex flex-col gap-2 px-3 py-3">
  {#if !hideLogoRow}
    <div class="flex items-center gap-2">
      <span class="text-lg">🐾</span>
      <span class="text-[13px] font-semibold text-foreground">PocketPaw</span>
    </div>
  {/if}

  {#if !isFilesMode}
    <Button onclick={newChat} variant="outline" size="sm" class={btnClass}>
      <Plus class="h-3.5 w-3.5" strokeWidth={2} />
      New Chat
    </Button>
  {/if}
</div>
```

Note: The `goto` target changes from `"/"` (which is now Files) to `"/chat"`. This is important since `/` is the explorer.

**Step 2: Also update SessionItem.svelte's `handleClick` to navigate to `/chat` instead of `/`**

In `SessionItem.svelte`, change:
```typescript
if (page.url.pathname !== "/") {
  goto("/");
}
```
to:
```typescript
if (page.url.pathname !== "/chat") {
  goto("/chat");
}
```

**Step 3: Verify it builds**

Run: `cd client && bun run check`

**Step 4: Commit**

```bash
git add client/src/lib/components/SidebarHeader.svelte client/src/lib/components/SessionItem.svelte
git commit -m "feat(sidebar): make header mode-aware, fix chat navigation to /chat"
```

---

### Task 8: Final Build Check and Polish

**Files:**
- All modified files

**Step 1: Full type check**

Run: `cd client && bun run check`

Fix any TypeScript errors.

**Step 2: Visual review**

Run: `cd client && bun run dev`

Verify:
- On `/` (Files): Left sidebar shows Quick Access, Locations, Cloud Storage sections
- On `/chat`: Left sidebar shows search, filter chips, pinned sessions, date-grouped sessions
- On `/settings` or any tool page: Same as `/chat`
- Cloud storage items show "Coming soon" tooltip and are visually disabled
- Filter chips toggle correctly
- Star/pin buttons appear on session items
- Pinned sessions show at the top of the list

**Step 3: Commit any polish fixes**

```bash
git add -A
git commit -m "fix(sidebar): polish mode-based sidebar styling and interactions"
```

---

## Important Notes

**Route check:** The current app uses `/` as the Files/Explorer default route and `/chat` for chat (per WorkspaceTabs). Session clicks currently navigate to `/` which needs updating to `/chat` (Task 7 covers this).

**Existing components preserved:** `SidebarFooter.svelte` is unchanged and shared. `SessionSearch.svelte` is unchanged.

**Store reuse:** `explorerStore.defaultDirs` and `explorerStore.pinnedFolders` already exist. `connectionStore.getClient().getRecentFiles()` already exists. No new backend API needed.

**shadcn Tooltip:** Already available in the project at `$lib/components/ui/tooltip`. If not present, install via `bunx shadcn-svelte@latest add tooltip`.

**No tests configured:** Per `client/CLAUDE.md`, there are no test commands configured for the client. Verification is via `bun run check` (type checking) and manual visual testing.
