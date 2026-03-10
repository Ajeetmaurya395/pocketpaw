# Backend-Aware Tool Bridge Exclusion

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Stop excluding shell/filesystem tools from the tool bridge for non-Claude backends, so `openai_agents`, `google_adk`, `codex_cli`, `copilot_sdk`, and `opencode` get full autonomous tool access without requiring the Claude Code CLI.

**Architecture:** Add a `backend` parameter (default `"claude_agent_sdk"`) to `_instantiate_all_tools()` and all public bridge functions. When `backend == "claude_agent_sdk"`, the current `_EXCLUDED_TOOLS` set applies (those tools are SDK-native). For any other backend, only `BrowserTool` and `DesktopTool` remain excluded (they require special session state / desktop access that the bridge can't provide).

**Tech Stack:** Python, pytest, unittest.mock

---

### Task 1: Update `_instantiate_all_tools()` to accept a backend parameter

**Files:**
- Modify: `src/pocketpaw/agents/tool_bridge.py:22-55`

**Step 1: Update the exclusion sets and function signature**

Replace lines 22-55 of `tool_bridge.py` with:

```python
# Tools excluded from ALL backends — need special session state or desktop access.
_ALWAYS_EXCLUDED = frozenset({"BrowserTool", "DesktopTool"})

# Tools excluded only for claude_agent_sdk — these are provided natively by the CLI.
_CLAUDE_SDK_EXCLUDED = frozenset(
    {"ShellTool", "ReadFileTool", "WriteFileTool", "ListDirTool"}
)


def _instantiate_all_tools(backend: str = "claude_agent_sdk") -> list[BaseTool]:
    """Discover and instantiate all builtin tools, filtered by backend.

    Args:
        backend: The agent backend name. For ``claude_agent_sdk``, shell/fs
                 tools are excluded (they're SDK builtins). Other backends
                 get the full set minus browser/desktop.

    Returns a list of BaseTool instances.  Import errors per-tool are caught
    and logged so one broken tool doesn't block the rest.
    """
    from pocketpaw.tools.builtin import _LAZY_IMPORTS

    excluded = _ALWAYS_EXCLUDED
    if backend == "claude_agent_sdk":
        excluded = excluded | _CLAUDE_SDK_EXCLUDED

    tools: list[BaseTool] = []
    for class_name, (module_path, attr_name) in _LAZY_IMPORTS.items():
        if class_name in excluded:
            continue
        try:
            import importlib

            mod = importlib.import_module(module_path, "pocketpaw.tools.builtin")
            cls = getattr(mod, attr_name)
            tools.append(cls())
        except Exception as exc:
            logger.debug("Skipping tool %s: %s", class_name, exc)
    return tools
```

**Step 2: Run existing tests to confirm they still import correctly**

Run: `cd /d/backend && uv run pytest tests/test_tool_bridge.py -v -x 2>&1 | head -40`
Expected: Some tests may fail because the old `_EXCLUDED_TOOLS` constant is gone. That's expected, we'll fix the tests in Task 3.

---

### Task 2: Thread `backend` parameter through all public bridge functions

**Files:**
- Modify: `src/pocketpaw/agents/tool_bridge.py:58-259`

**Step 1: Update `build_openai_function_tools`**

Change signature and body (line 58):

```python
def build_openai_function_tools(settings: Any, backend: str = "openai_agents") -> list:
```

And update the call to `_instantiate_all_tools` inside it (line 85):

```python
    for tool in _instantiate_all_tools(backend=backend):
```

**Step 2: Update `build_adk_function_tools`**

Change signature (line 128):

```python
def build_adk_function_tools(settings: Any, backend: str = "google_adk") -> list:
```

And update the call (line 156):

```python
    for tool in _instantiate_all_tools(backend=backend):
```

**Step 3: Update `get_tool_instructions_compact`**

Change signature (line 216):

```python
def get_tool_instructions_compact(settings: Any, backend: str = "opencode") -> list:
```

And update the call (line 237):

```python
    for tool in _instantiate_all_tools(backend=backend):
```

**Step 4: Update the module docstring (line 1-8)**

```python
"""Tool bridge -- adapts PocketPaw tools for use by different agent backends.

Provides:
- _instantiate_all_tools(backend): discover and instantiate builtin tools, filtered by backend
- build_openai_function_tools(): wrap tools as OpenAI Agents SDK FunctionTool objects
- build_adk_function_tools(): wrap tools as Google ADK FunctionTool objects
- get_tool_instructions_compact(): compact markdown for system-prompt injection

Backend-aware exclusion:
- claude_agent_sdk: shell/fs tools excluded (provided natively by CLI)
- All other backends: shell/fs tools included via the bridge
- BrowserTool/DesktopTool: always excluded (need special session state)
"""
```

---

### Task 3: Update tests for backend-aware exclusion

**Files:**
- Modify: `tests/test_tool_bridge.py`

**Step 1: Update `test_excludes_shell_and_filesystem` to test claude_agent_sdk behavior**

```python
    def test_excludes_shell_and_filesystem_for_claude_sdk(self):
        from pocketpaw.agents.tool_bridge import _instantiate_all_tools

        tools = _instantiate_all_tools(backend="claude_agent_sdk")
        names = {type(t).__name__ for t in tools}
        assert "ShellTool" not in names
        assert "ReadFileTool" not in names
        assert "WriteFileTool" not in names
        assert "ListDirTool" not in names
```

**Step 2: Update `test_excludes_browser_and_desktop` to verify always-excluded**

```python
    def test_excludes_browser_and_desktop_always(self):
        from pocketpaw.agents.tool_bridge import _instantiate_all_tools

        # Should be excluded for ALL backends
        for backend in ["claude_agent_sdk", "openai_agents", "google_adk"]:
            tools = _instantiate_all_tools(backend=backend)
            names = {type(t).__name__ for t in tools}
            assert "BrowserTool" not in names, f"BrowserTool included for {backend}"
            assert "DesktopTool" not in names, f"DesktopTool included for {backend}"
```

**Step 3: Add new test for non-Claude backends getting shell/fs tools**

```python
    def test_includes_shell_and_filesystem_for_other_backends(self):
        from pocketpaw.agents.tool_bridge import _instantiate_all_tools

        for backend in ["openai_agents", "google_adk", "codex_cli", "copilot_sdk"]:
            tools = _instantiate_all_tools(backend=backend)
            names = {type(t).__name__ for t in tools}
            assert "ShellTool" in names, f"ShellTool missing for {backend}"
            assert "ReadFileTool" in names, f"ReadFileTool missing for {backend}"
            assert "WriteFileTool" in names, f"WriteFileTool missing for {backend}"
            assert "ListDirTool" in names, f"ListDirTool missing for {backend}"
```

**Step 4: Run all tool bridge tests**

Run: `cd /d/backend && uv run pytest tests/test_tool_bridge.py -v`
Expected: All PASS

---

### Task 4: Run full test suite and lint

**Files:** None (verification only)

**Step 1: Run linter**

Run: `cd /d/backend && uv run ruff check src/pocketpaw/agents/tool_bridge.py tests/test_tool_bridge.py`
Expected: No errors

**Step 2: Run full test suite (excluding E2E)**

Run: `cd /d/backend && uv run pytest --ignore=tests/e2e -v -x 2>&1 | tail -30`
Expected: All PASS

---

### Task 5: Commit

**Step 1: Stage and commit**

```bash
git add src/pocketpaw/agents/tool_bridge.py tests/test_tool_bridge.py
git commit -m "fix: include shell/fs tools in tool bridge for non-Claude backends

The tool bridge excluded ShellTool, ReadFileTool, WriteFileTool, and
ListDirTool from all backends. This was only correct for claude_agent_sdk
(which provides them natively via the CLI subprocess). Other backends
(openai_agents, google_adk, codex_cli, copilot_sdk, opencode) need these
tools from the bridge to provide full autonomous shell/filesystem access.

Add a backend parameter to _instantiate_all_tools() and all public bridge
functions. Only claude_agent_sdk excludes shell/fs tools. BrowserTool and
DesktopTool remain excluded for all backends (need special session state)."
```
