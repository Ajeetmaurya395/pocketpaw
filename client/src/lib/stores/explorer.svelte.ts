import type { FileEntry, DefaultDirs, FileChangeEvent } from "$lib/filesystem";
import { localFs, joinPath, getFileName, invalidateThumbnail, isImageFile } from "$lib/filesystem";

export interface Breadcrumb {
  name: string;
  path: string;
}

export interface PinnedFolder {
  path: string;
  name: string;
  source: string;
}

const PINNED_KEY = "pocketpaw_pinned_folders";
const VIEW_KEY = "pocketpaw_explorer_view";

class ExplorerStore {
  currentPath = $state("");
  currentSource = $state<"local" | "remote" | "cloud">("local");
  files = $state<FileEntry[]>([]);
  selectedFiles = $state<Set<string>>(new Set());
  viewMode = $state<"icon" | "grid" | "list" | "column" | "gallery">("icon");
  sortBy = $state<"name" | "modified" | "size" | "type">("name");
  sortAsc = $state(true);
  isLoading = $state(false);
  error = $state<string | null>(null);
  history = $state<string[]>([]);
  historyIndex = $state(-1);
  openFile = $state<FileEntry | null>(null);
  chatSidebarOpen = $state(true);
  defaultDirs = $state<DefaultDirs | null>(null);
  pinnedFolders = $state<PinnedFolder[]>([]);
  searchQuery = $state("");
  renamingFile = $state<string | null>(null);
  clipboardFiles = $state<Set<string>>(new Set());
  clipboardMode = $state<"copy" | "cut" | null>(null);
  focusedIndex = $state(-1);
  openFileChangedOnDisk = $state(0);

  private unwatchFn: (() => void) | null = null;
  private debounceTimer: ReturnType<typeof setTimeout> | null = null;

  filteredFiles = $derived.by(() => {
    if (!this.searchQuery.trim()) return this.files;
    const q = this.searchQuery.trim().toLowerCase();
    return this.files.filter((f) => f.name.toLowerCase().includes(q));
  });

  sortedFiles = $derived.by(() => {
    const sorted = [...this.filteredFiles].sort((a, b) => {
      // Directories always first
      if (a.isDir !== b.isDir) return a.isDir ? -1 : 1;

      let cmp = 0;
      switch (this.sortBy) {
        case "name":
          cmp = a.name.localeCompare(b.name, undefined, { sensitivity: "base" });
          break;
        case "modified":
          cmp = a.modified - b.modified;
          break;
        case "size":
          cmp = a.size - b.size;
          break;
        case "type":
          cmp = a.extension.localeCompare(b.extension);
          break;
      }
      return this.sortAsc ? cmp : -cmp;
    });
    return sorted;
  });

  isHome = $derived(this.currentPath === "");
  isDetailView = $derived(this.openFile !== null);
  canGoBack = $derived(this.historyIndex > 0);
  canGoForward = $derived(this.historyIndex < this.history.length - 1);

  breadcrumbs = $derived.by((): Breadcrumb[] => {
    if (!this.currentPath) return [];
    // Handle both Windows and Unix paths
    const normalized = this.currentPath.replace(/\\/g, "/");
    const parts = normalized.split("/").filter(Boolean);
    const crumbs: Breadcrumb[] = [];

    // Handle drive letter on Windows (e.g., "C:")
    let accumulated = "";
    for (let i = 0; i < parts.length; i++) {
      if (i === 0 && parts[0].endsWith(":")) {
        accumulated = parts[0] + "/";
      } else {
        accumulated += (accumulated.endsWith("/") ? "" : "/") + parts[i];
      }
      crumbs.push({ name: parts[i], path: accumulated });
    }
    return crumbs;
  });

  async initialize(): Promise<void> {
    // Load persisted view mode
    const savedView = localStorage.getItem(VIEW_KEY);
    if (savedView) {
      this.viewMode = savedView as typeof this.viewMode;
    }

    // Load pinned folders
    try {
      const raw = localStorage.getItem(PINNED_KEY);
      if (raw) {
        this.pinnedFolders = JSON.parse(raw);
      }
    } catch {
      // Ignore parse errors
    }

    // Load default dirs
    try {
      this.defaultDirs = await localFs.getDefaultDirs();
    } catch {
      // Not in Tauri or error — leave null
    }
  }

  async navigateTo(path: string, source?: "local" | "remote" | "cloud"): Promise<void> {
    this.isLoading = true;
    this.error = null;
    this.openFile = null;
    this.selectedFiles = new Set();
    this.searchQuery = "";
    this.focusedIndex = -1;

    try {
      const s = source ?? this.currentSource;
      const entries = await localFs.readDir(path);
      this.files = entries;
      this.currentPath = path;
      this.currentSource = s;

      // Push to history
      if (this.historyIndex < this.history.length - 1) {
        this.history = this.history.slice(0, this.historyIndex + 1);
      }
      this.history = [...this.history, path];
      this.historyIndex = this.history.length - 1;

      // Start watching
      await this.startWatching(path);
    } catch (e) {
      this.error = e instanceof Error ? e.message : String(e);
    } finally {
      this.isLoading = false;
    }
  }

  async refresh(): Promise<void> {
    if (!this.currentPath) return;
    this.isLoading = true;
    this.error = null;
    try {
      this.files = await localFs.readDir(this.currentPath);
    } catch (e) {
      this.error = e instanceof Error ? e.message : String(e);
    } finally {
      this.isLoading = false;
    }
  }

  goBack(): void {
    if (!this.canGoBack) return;
    this.historyIndex--;
    const path = this.history[this.historyIndex];
    this.loadWithoutHistory(path);
  }

  goForward(): void {
    if (!this.canGoForward) return;
    this.historyIndex++;
    const path = this.history[this.historyIndex];
    this.loadWithoutHistory(path);
  }

  goHome(): void {
    this.currentPath = "";
    this.files = [];
    this.openFile = null;
    this.selectedFiles = new Set();
    this.searchQuery = "";
    this.focusedIndex = -1;
    this.error = null;
    this.stopWatching();
  }

  openFileDetail(file: FileEntry): void {
    this.openFile = file;
  }

  closeDetail(): void {
    this.openFile = null;
  }

  selectFile(path: string, multi = false): void {
    if (multi) {
      const next = new Set(this.selectedFiles);
      if (next.has(path)) {
        next.delete(path);
      } else {
        next.add(path);
      }
      this.selectedFiles = next;
    } else {
      this.selectedFiles = new Set([path]);
    }
  }

  selectAll(): void {
    this.selectedFiles = new Set(this.files.map((f) => f.path));
  }

  clearSelection(): void {
    this.selectedFiles = new Set();
  }

  copyToClipboard(): void {
    if (this.selectedFiles.size === 0) return;
    this.clipboardFiles = new Set(this.selectedFiles);
    this.clipboardMode = "copy";
  }

  cutToClipboard(): void {
    if (this.selectedFiles.size === 0) return;
    this.clipboardFiles = new Set(this.selectedFiles);
    this.clipboardMode = "cut";
  }

  async paste(): Promise<void> {
    if (this.clipboardFiles.size === 0 || !this.clipboardMode || !this.currentPath) return;
    try {
      for (const srcPath of this.clipboardFiles) {
        const name = getFileName(srcPath);
        const destPath = joinPath(this.currentPath, name);
        const stat = await localFs.stat(srcPath);
        if (this.clipboardMode === "copy") {
          if (stat.isDir) {
            await localFs.copyDir(srcPath, destPath);
          } else {
            await localFs.copyFile(srcPath, destPath);
          }
        } else {
          await localFs.moveFile(srcPath, destPath);
        }
      }
      if (this.clipboardMode === "cut") {
        this.clipboardFiles = new Set();
        this.clipboardMode = null;
      }
      await this.refresh();
    } catch (e) {
      console.error("Paste failed:", e);
    }
  }

  moveFocus(delta: number): void {
    const files = this.sortedFiles;
    if (files.length === 0) return;
    let next = this.focusedIndex + delta;
    if (next < 0) next = 0;
    if (next >= files.length) next = files.length - 1;
    this.focusedIndex = next;
    this.selectedFiles = new Set([files[next].path]);
  }

  setViewMode(mode: typeof this.viewMode): void {
    this.viewMode = mode;
    localStorage.setItem(VIEW_KEY, mode);
  }

  setSortBy(field: typeof this.sortBy): void {
    if (this.sortBy === field) {
      this.sortAsc = !this.sortAsc;
    } else {
      this.sortBy = field;
      this.sortAsc = true;
    }
  }

  toggleChatSidebar(): void {
    this.chatSidebarOpen = !this.chatSidebarOpen;
  }

  setSearchQuery(query: string): void {
    this.searchQuery = query;
  }

  startRename(filePath: string): void {
    this.renamingFile = filePath;
  }

  cancelRename(): void {
    this.renamingFile = null;
  }

  async commitRename(oldPath: string, newName: string): Promise<void> {
    if (!newName.trim() || newName.includes("/") || newName.includes("\\")) {
      this.renamingFile = null;
      return;
    }
    try {
      const { parentDir, joinPath } = await import("$lib/filesystem");
      const dir = parentDir(oldPath);
      const newPath = joinPath(dir, newName);
      await localFs.rename(oldPath, newPath);
      this.renamingFile = null;
      await this.refresh();
    } catch (e) {
      console.error("Rename failed:", e);
      this.renamingFile = null;
    }
  }

  async createFolder(name: string): Promise<void> {
    if (!this.currentPath || !name.trim()) return;
    try {
      const { joinPath } = await import("$lib/filesystem");
      const newPath = joinPath(this.currentPath, name);
      await localFs.createDir(newPath);
      await this.refresh();
    } catch (e) {
      console.error("Create folder failed:", e);
    }
  }

  pinFolder(path: string, name: string, source: string): void {
    if (this.pinnedFolders.some((p) => p.path === path)) return;
    this.pinnedFolders = [...this.pinnedFolders, { path, name, source }];
    localStorage.setItem(PINNED_KEY, JSON.stringify(this.pinnedFolders));
  }

  unpinFolder(path: string): void {
    this.pinnedFolders = this.pinnedFolders.filter((p) => p.path !== path);
    localStorage.setItem(PINNED_KEY, JSON.stringify(this.pinnedFolders));
  }

  private async loadWithoutHistory(path: string): Promise<void> {
    this.isLoading = true;
    this.error = null;
    this.openFile = null;
    this.selectedFiles = new Set();
    this.searchQuery = "";
    this.focusedIndex = -1;
    try {
      this.files = await localFs.readDir(path);
      this.currentPath = path;
      await this.startWatching(path);
    } catch (e) {
      this.error = e instanceof Error ? e.message : String(e);
    } finally {
      this.isLoading = false;
    }
  }

  private debouncedRefresh(): void {
    if (this.debounceTimer) clearTimeout(this.debounceTimer);
    this.debounceTimer = setTimeout(() => {
      this.debounceTimer = null;
      this.refresh();
    }, 400);
  }

  private async startWatching(path: string): Promise<void> {
    this.stopWatching();
    try {
      this.unwatchFn = await localFs.watch(path, (event: FileChangeEvent) => {
        const eventPath = event.path.replace(/\\/g, "/");

        // On image modify, invalidate its thumbnail
        if (event.kind === "modify" && !event.is_dir) {
          const ext = eventPath.split(".").pop()?.toLowerCase() ?? "";
          if (isImageFile(ext)) {
            invalidateThumbnail(event.path);
          }
        }

        // If the open file was modified, signal the viewer
        if (event.kind === "modify" && this.openFile) {
          const openPath = this.openFile.path.replace(/\\/g, "/");
          if (eventPath === openPath) {
            this.openFileChangedOnDisk++;
          }
        }

        // Always debounce-refresh the directory listing
        this.debouncedRefresh();
      });
    } catch {
      // Watching not available
    }
  }

  private stopWatching(): void {
    if (this.debounceTimer) {
      clearTimeout(this.debounceTimer);
      this.debounceTimer = null;
    }
    if (this.unwatchFn) {
      this.unwatchFn();
      this.unwatchFn = null;
    }
  }
}

export const explorerStore = new ExplorerStore();
