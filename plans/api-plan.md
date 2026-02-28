# PawKits & Command Centers — Backend API Plan

## Overview

Backend infrastructure for PawKits: Pydantic models, file-based store, REST API endpoints, WebSocket events, and auto-install of built-in kits.

---

## Step 1: Pydantic Models

**New file:** `src/pocketpaw/kits/__init__.py`
**New file:** `src/pocketpaw/kits/models.py`

Pydantic equivalents of the TypeScript types. Follows the same pattern as `mission_control/models.py`.

### Models

```python
class PawKitMeta(BaseModel):
    name: str
    author: str
    version: str
    description: str
    category: str
    tags: list[str]
    icon: str
    built_in: bool = False

class MetricItem(BaseModel):
    label: str
    source: str           # "workflow:<id>" or "api:<endpoint>"
    field: str
    format: Literal["number", "currency", "percent", "text"]
    trend: bool = False

class PanelConfig(BaseModel):
    id: str
    type: Literal["metrics-row", "table", "kanban", "chart", "feed", "markdown"]
    model_config = ConfigDict(extra="allow")  # type-specific config

class SectionConfig(BaseModel):
    title: str
    span: Literal["full", "left", "right"]
    panels: list[PanelConfig]

class LayoutConfig(BaseModel):
    columns: int
    sections: list[SectionConfig]

class WorkflowConfig(BaseModel):
    schedule: str | None = None
    trigger: dict | None = None
    instruction: str
    output_type: Literal["structured", "feed", "task_list", "document"]
    retry: int | None = None

class UserConfigField(BaseModel):
    key: str
    label: str
    type: Literal["text", "secret", "select", "number"]
    placeholder: str | None = None
    options: list[str] | None = None
    help_url: str | None = None

class PawKitConfig(BaseModel):
    meta: PawKitMeta
    layout: LayoutConfig
    workflows: dict[str, WorkflowConfig] = {}
    user_config: list[UserConfigField] | None = None
    skills: list[str] | None = None
    integrations: dict | None = None

class InstalledKit(BaseModel):
    id: str
    config: PawKitConfig
    user_values: dict[str, str] = {}
    installed_at: str
    active: bool = False
```

---

## Step 2: File-based Kit Store

**New file:** `src/pocketpaw/kits/store.py`

Follows the `FileMissionControlStore` pattern:

### Storage Layout
```
~/.pocketpaw/kits/
  <kit-id>/
    pawkit.yaml    — the config
    data/          — workflow output JSON files
    skills/        — bundled .md files
```

### Interface
```python
class FileKitStore:
    def __init__(self, base_dir: Path | None = None)

    async def install_kit(self, yaml_str: str, kit_id: str | None = None) -> InstalledKit
    async def get_kit(self, kit_id: str) -> InstalledKit | None
    async def list_kits(self) -> list[InstalledKit]
    async def remove_kit(self, kit_id: str) -> bool
    async def activate_kit(self, kit_id: str) -> bool
    async def get_kit_data(self, kit_id: str) -> dict[str, Any]
    async def save_kit_data(self, kit_id: str, source: str, data: Any) -> None

# Singleton factory
_store: FileKitStore | None = None
def get_kit_store() -> FileKitStore
```

### Key Details
- In-memory index loaded on first access
- Atomic writes (write to temp file, then rename)
- `install_kit()` validates YAML against `PawKitConfig`, generates slug ID from name
- `activate_kit()` deactivates all other kits first
- `get_kit_data()` resolves `api:*` sources by calling Mission Control store

---

## Step 3: Built-in "Project Orchestrator" Kit

**New file:** `src/pocketpaw/kits/builtins/__init__.py`
**New file:** `src/pocketpaw/kits/builtins/project_orchestrator.yaml`

```yaml
meta:
  name: "Project Orchestrator"
  author: "PocketPaw"
  version: "1.0.0"
  description: "Track tasks, monitor agent progress, and manage your projects"
  category: general
  icon: layout-dashboard
  tags: [tasks, projects, orchestration]
  built_in: true

layout:
  columns: 2
  sections:
    - title: "Overview"
      span: full
      panels:
        - id: metrics
          type: metrics-row
          items:
            - { label: "Total Tasks", source: "api:stats", field: "tasks.total", format: "number" }
            - { label: "In Progress", source: "api:stats", field: "tasks.by_status.in_progress", format: "number" }
            - { label: "Completed", source: "api:stats", field: "tasks.by_status.done", format: "number" }
            - { label: "Blocked", source: "api:stats", field: "tasks.by_status.blocked", format: "number" }

    - title: "Task Board"
      span: full
      panels:
        - id: kanban
          type: kanban
          columns:
            - { key: "inbox", label: "Inbox", color: "gray" }
            - { key: "in_progress", label: "In Progress", color: "blue" }
            - { key: "review", label: "Review", color: "orange" }
            - { key: "done", label: "Done", color: "green" }
          source: "api:tasks"
          card_fields: [title, priority, task_type]

    - title: "Recent Activity"
      span: left
      panels:
        - id: activity_feed
          type: feed
          source: "api:activities"
          max_items: 20

    - title: "Documents"
      span: right
      panels:
        - id: docs
          type: table
          source: "api:documents"
          columns:
            - { key: "title", label: "Title" }
            - { key: "type", label: "Type" }
            - { key: "updated_at", label: "Updated" }

workflows: {}
```

---

## Step 4: REST API Endpoints

**New file:** `src/pocketpaw/api/v1/kits.py`

**Modify:** `src/pocketpaw/api/v1/__init__.py` — register in `_V1_ROUTERS`:
```python
("pocketpaw.api.v1.kits", "router", "Kits"),
```

### Endpoints

| Method | Path | Description |
|--------|------|-------------|
| `GET /kits` | List installed kits | Returns `InstalledKit[]` |
| `GET /kits/{kit_id}` | Get kit config + data | Returns full `InstalledKit` |
| `POST /kits/install` | Install from YAML string | Validates, stores, returns kit ID |
| `DELETE /kits/{kit_id}` | Uninstall a kit | Removes dir |
| `GET /kits/{kit_id}/data` | Get all panel data | Returns `Record<source, data>` |
| `GET /kits/{kit_id}/data/{source}` | Get specific data source | Returns data for one source |
| `POST /kits/{kit_id}/activate` | Set as active kit | Marks as active Command Center |

### Data Resolution (in `/kits/{kit_id}/data`)

For the "Project Orchestrator" kit, `api:*` sources resolve to Mission Control store calls:

- `api:stats` → `get_mission_control_store().get_stats()`
- `api:tasks` → `get_mission_control_store().list_tasks()` → grouped by status for kanban
- `api:activities` → `get_mission_control_store().get_activity_feed()`
- `api:documents` → `get_mission_control_store().list_documents()`

Response shape:
```json
{
  "api:stats": { "tasks": { "total": 24, "by_status": { ... } } },
  "api:tasks": { "inbox": [...], "in_progress": [...] },
  "api:activities": [...],
  "api:documents": [...]
}
```

---

## Step 5: Auto-install on Startup

**Modify:** `src/pocketpaw/dashboard_lifecycle.py`

On backend startup, check if "Project Orchestrator" kit exists. If not, auto-install from bundled YAML and activate it.

```python
async def _ensure_builtin_kits():
    store = get_kit_store()
    kits = await store.list_kits()
    builtin_ids = {k.id for k in kits if k.config.meta.built_in}
    if "project-orchestrator" not in builtin_ids:
        yaml_path = Path(__file__).parent / "kits" / "builtins" / "project_orchestrator.yaml"
        yaml_str = yaml_path.read_text()
        kit = await store.install_kit(yaml_str, kit_id="project-orchestrator")
        await store.activate_kit(kit.id)
```

---

## Step 6: WebSocket Events (Phase 1 — Polling Only)

For Phase 1, the client polls every 30s. No WS instrumentation needed yet.

Future WS event type (for Phase 1.5):
```python
{
    "type": "kit_data_update",
    "kit_id": kit_id,
    "source": "api:tasks",
    "data": updated_data
}
```

---

## Implementation Order

1. `kits/__init__.py` + `kits/models.py` — Pydantic models
2. `kits/store.py` — File-based store with singleton
3. `kits/builtins/project_orchestrator.yaml` — Built-in kit YAML
4. `api/v1/kits.py` + register in `__init__.py` — REST endpoints
5. `dashboard_lifecycle.py` — Auto-install hook

---

## Verification

- `uv run pytest` — add tests for kit store CRUD + API endpoints
- `uv run ruff check .` — lint passes
- `uv run mypy .` — type check passes
- Manual: `GET /api/v1/kits` returns the auto-installed Project Orchestrator
