# PawKits & Command Centers — Client Plan

## Overview

Client-side infrastructure for PawKits: TypeScript types, Svelte 5 store, REST/WS integration, panel components, layout renderer, and Command Center route.

---

## Step 1: TypeScript Types

**New file:** `client/src/lib/types/pawkit.ts`

```typescript
export interface PawKitMeta {
  name: string;
  author: string;
  version: string;
  description: string;
  category: string;
  tags: string[];
  icon: string;
  built_in?: boolean;
}

export interface MetricItem {
  label: string;
  source: string;       // "workflow:<id>" or "api:<endpoint>"
  field: string;
  format: "number" | "currency" | "percent" | "text";
  trend?: boolean;
}

export interface PanelConfig {
  id: string;
  type: "metrics-row" | "table" | "kanban" | "chart" | "feed" | "markdown";
  [key: string]: unknown;  // type-specific config
}

export interface SectionConfig {
  title: string;
  span: "full" | "left" | "right";
  panels: PanelConfig[];
}

export interface LayoutConfig {
  columns: number;
  sections: SectionConfig[];
}

export interface WorkflowConfig {
  schedule?: string;
  trigger?: { type: string; source: string; condition: string };
  instruction: string;
  output_type: "structured" | "feed" | "task_list" | "document";
  retry?: number;
}

export interface UserConfigField {
  key: string;
  label: string;
  type: "text" | "secret" | "select" | "number";
  placeholder?: string;
  options?: string[];
  help_url?: string;
}

export interface PawKitConfig {
  meta: PawKitMeta;
  layout: LayoutConfig;
  workflows: Record<string, WorkflowConfig>;
  user_config?: UserConfigField[];
  skills?: string[];
  integrations?: {
    required?: string[];
    optional?: string[];
  };
}

export interface InstalledKit {
  id: string;
  config: PawKitConfig;
  user_values: Record<string, string>;
  installed_at: string;
  active: boolean;
}

export interface WorkflowData {
  workflow_id: string;
  kit_id: string;
  data: unknown;
  updated_at: string;
}
```

---

## Step 2: Add `js-yaml` Dependency

```bash
cd client && bun add js-yaml && bun add -d @types/js-yaml
```

Used for client-side YAML parsing (local preview). Backend uses Python's `pyyaml`.

---

## Step 3: API Integration

### 3a. Add WS event type

**Modify:** `client/src/lib/api/types.ts`

```typescript
export interface WSKitDataUpdate {
  type: "kit_data_update";
  kit_id: string;
  source: string;
  data: unknown;
}
```

Add to `WSEvent` union type.

### 3b. Add REST methods

**Modify:** `client/src/lib/api/client.ts`

```typescript
// Kit methods
async listKits(): Promise<InstalledKit[]>
async getKit(kitId: string): Promise<InstalledKit>
async installKit(yaml: string): Promise<{ id: string }>
async removeKit(kitId: string): Promise<void>
async getKitData(kitId: string): Promise<Record<string, unknown>>
async activateKit(kitId: string): Promise<void>
```

---

## Step 4: Kit Store (Svelte 5 Runes)

**New file:** `client/src/lib/stores/kits.svelte.ts`

Follows the `SkillStore` pattern (`stores/skills.svelte.ts`):

```typescript
class KitStore {
  kits = $state<InstalledKit[]>([]);
  activeKitId = $state<string | null>(null);
  activeKit = $derived(this.kits.find(k => k.id === this.activeKitId) ?? null);
  kitData = $state<Record<string, unknown>>({});
  isLoading = $state(false);

  private pollInterval: ReturnType<typeof setInterval> | null = null;

  async load(): Promise<void>           // fetch installed kits
  async activate(id: string): void      // set active
  async loadData(kitId: string): void   // fetch panel data
  async install(yaml: string): void     // install new kit
  async remove(kitId: string): void     // uninstall

  // Polling for data updates (30s interval)
  startPolling(kitId: string): void
  stopPolling(): void

  // Future: WebSocket binding for real-time data updates
  bindEvents(ws: PocketPawWebSocket): void
  disposeEvents(): void
}

export const kitStore = new KitStore();
```

**Modify:** `client/src/lib/stores/index.ts`

- Export `kitStore`
- Add to `initializeStores()`:
  ```typescript
  kitStore.bindEvents(ws);
  kitStore.load();  // background, non-blocking
  ```

---

## Step 5: Panel Components

**New directory:** `client/src/lib/components/panels/`

### 5a. MetricsRow.svelte

Row of KPI cards. Each card shows: label, value, optional trend arrow.

```
┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐
│ Total    │ │ Active   │ │ Done     │ │ Blocked  │
│ 24       │ │ 8        │ │ 12       │ │ 2        │
└──────────┘ └──────────┘ └──────────┘ └──────────┘
```

- Uses existing shadcn `Card` component
- Resolves `source` + `field` from `kitData` using dot-notation path (e.g., `tasks.by_status.done`)
- Props: `config: PanelConfig`, `data: unknown`

### 5b. KanbanBoard.svelte

Columns with cards. No drag-and-drop in Phase 1.

```
┌─ Inbox ──────┐ ┌─ In Progress ─┐ ┌─ Review ──────┐ ┌─ Done ────────┐
│ ┌──────────┐ │ │ ┌──────────┐  │ │ ┌──────────┐  │ │ ┌──────────┐  │
│ │ Task A   │ │ │ │ Task B   │  │ │ │ Task D   │  │ │ │ Task E   │  │
│ │ medium   │ │ │ │ high     │  │ │ │ low      │  │ │ │ medium   │  │
│ └──────────┘ │ │ └──────────┘  │ │ └──────────┘  │ │ └──────────┘  │
└──────────────┘ └───────────────┘ └──────────────┘ └───────────────┘
```

- CSS grid for columns
- Existing `Badge` component for priority
- `ScrollArea` for overflow
- Props: `config: PanelConfig`, `data: unknown`

### 5c. Feed.svelte

Chronological event list.

```
● Task "Analyze data" completed                    2m ago
● Agent assigned to "Write report"                 5m ago
● Document "Research Notes" created                12m ago
```

- Simple list with relative time formatting
- Uses existing `Avatar`/`Badge` components
- Props: `config: PanelConfig`, `data: unknown`

### 5d. DataTable.svelte

Sortable table with columns defined in panel config.

| Title | Type | Updated |
|-------|------|---------|
| Research Notes | research | 2 hours ago |
| Weekly Report | deliverable | 1 day ago |

- HTML `<table>` styled with Tailwind
- Click column header to sort
- No external dependencies
- Props: `config: PanelConfig`, `data: unknown`

### 5e. PanelRenderer.svelte (dispatcher)

Takes a `PanelConfig` + `data` and dispatches to the correct panel component:

```svelte
{#if panel.type === "metrics-row"}
  <MetricsRow config={panel} data={panelData} />
{:else if panel.type === "kanban"}
  <KanbanBoard config={panel} data={panelData} />
{:else if panel.type === "feed"}
  <Feed config={panel} data={panelData} />
{:else if panel.type === "table"}
  <DataTable config={panel} data={panelData} />
{:else}
  <div class="text-muted-foreground text-sm">Unknown panel: {panel.type}</div>
{/if}
```

---

## Step 6: Layout Renderer & Command Center View

### 6a. LayoutRenderer.svelte

**New file:** `client/src/lib/components/command-center/LayoutRenderer.svelte`

Takes `LayoutConfig` + `kitData`, renders sections → panels in CSS grid.

```svelte
<div class="grid grid-cols-2 gap-4 p-4">
  {#each config.sections as section}
    <div class={sectionSpanClass(section.span)}>
      <h3 class="text-sm font-medium text-foreground mb-2">{section.title}</h3>
      {#each section.panels as panel}
        <PanelRenderer {panel} data={resolveData(panel, kitData)} />
      {/each}
    </div>
  {/each}
</div>
```

`sectionSpanClass()`:
- `"full"` → `"col-span-2"`
- `"left"` → `"col-span-1"`
- `"right"` → `"col-span-1"`

Responsive: on mobile (`< 640px`), everything stacks (`col-span-2`).

### 6b. Command Center Page

**New file:** `client/src/routes/command-center/+page.svelte`

Main view:
1. Reads `kitStore.activeKit` config
2. Fetches panel data via `kitStore.loadData()`
3. Renders `LayoutRenderer` with config + data
4. Shows empty state if no kit installed
5. Starts 30s polling for data updates

```svelte
<script lang="ts">
  import { kitStore } from "$lib/stores";
  import LayoutRenderer from "$lib/components/command-center/LayoutRenderer.svelte";
  import EmptyCommandCenter from "$lib/components/command-center/EmptyState.svelte";
  import { onMount, onDestroy } from "svelte";

  let kit = $derived(kitStore.activeKit);
  let data = $derived(kitStore.kitData);
  let isLoading = $derived(kitStore.isLoading);

  onMount(() => {
    if (kit) {
      kitStore.loadData(kit.id);
      kitStore.startPolling(kit.id);
    }
  });

  onDestroy(() => {
    kitStore.stopPolling();
  });
</script>

{#if isLoading}
  <LoadingSpinner />
{:else if kit}
  <LayoutRenderer config={kit.config.layout} kitData={data} />
{:else}
  <EmptyCommandCenter />
{/if}
```

### 6c. EmptyState.svelte

**New file:** `client/src/lib/components/command-center/EmptyState.svelte`

Shows when no Command Center is active:
- "No Command Center active"
- "Install the Project Orchestrator to get started" button
- Button triggers auto-install of built-in kit

---

## Step 7: Navigation

**Modify:** `client/src/lib/components/SidebarFooter.svelte`

Add Command Center link:
```typescript
import { LayoutDashboard } from "@lucide/svelte";
// Add to tools grid:
{ href: "/command-center", label: "Command Center", icon: LayoutDashboard }
```

---

## Implementation Order

1. `client/src/lib/types/pawkit.ts` — TypeScript types
2. `bun add js-yaml @types/js-yaml` — YAML parser dependency
3. `client/src/lib/api/types.ts` — Add WS event type
4. `client/src/lib/api/client.ts` — Add REST methods
5. `client/src/lib/stores/kits.svelte.ts` — Kit store
6. `client/src/lib/stores/index.ts` — Export + initialize
7. Panel components (MetricsRow, KanbanBoard, Feed, DataTable, PanelRenderer)
8. Layout renderer + Command Center page + EmptyState
9. SidebarFooter navigation link

---

## Verification

- `cd client && bun run check` — TypeScript/Svelte type check passes
- Manual: Navigate to `/command-center`, see empty state or rendered dashboard
- Manual: Metrics, kanban, feed, and table panels render with data from backend

---

## What's NOT in Phase 1 (Client)

- Chart panel (needs Chart.js)
- Drag-and-drop on kanban
- Kit browse/marketplace UI
- Kit builder/conversation flow
- Markdown panel (trivial to add later)
