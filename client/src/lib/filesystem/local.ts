import type { FileEntry, DefaultDirs, FileChangeEvent, FileSystemProvider } from "./types";
import { getThumbnail as getCachedThumbnail } from "./thumbnail-cache";

/** Raw shape returned by the Rust fs_read_dir command */
interface RawFileEntry {
  name: string;
  path: string;
  is_dir: boolean;
  size: number;
  modified: number;
  extension: string;
}

function isTauri(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

function mapEntry(raw: RawFileEntry): FileEntry {
  return {
    name: raw.name,
    path: raw.path,
    isDir: raw.is_dir,
    size: raw.size,
    modified: raw.modified,
    extension: raw.extension,
    source: "local",
  };
}

export class LocalFileSystem implements FileSystemProvider {
  scheme = "local" as const;

  async readDir(path: string): Promise<FileEntry[]> {
    if (!isTauri()) return [];
    const { invoke } = await import("@tauri-apps/api/core");
    const raw: RawFileEntry[] = await invoke("fs_read_dir", { path });
    return raw.map(mapEntry);
  }

  async readFileText(path: string): Promise<string> {
    if (!isTauri()) return "";
    const { invoke } = await import("@tauri-apps/api/core");
    return invoke("fs_read_file_text", { path });
  }

  async writeFile(path: string, content: string): Promise<void> {
    if (!isTauri()) return;
    const { invoke } = await import("@tauri-apps/api/core");
    await invoke("fs_write_file", { path, content });
  }

  async deleteFile(path: string, recursive = false): Promise<void> {
    if (!isTauri()) return;
    const { invoke } = await import("@tauri-apps/api/core");
    await invoke("fs_delete", { path, recursive });
  }

  async rename(oldPath: string, newPath: string): Promise<void> {
    if (!isTauri()) return;
    const { invoke } = await import("@tauri-apps/api/core");
    await invoke("fs_rename", { oldPath, newPath });
  }

  async stat(path: string): Promise<FileEntry> {
    if (!isTauri()) {
      return { name: "", path, isDir: false, size: 0, modified: 0, extension: "", source: "local" };
    }
    const { invoke } = await import("@tauri-apps/api/core");
    const raw: RawFileEntry = await invoke("fs_stat", { path });
    return mapEntry(raw);
  }

  async createDir(path: string): Promise<void> {
    if (!isTauri()) return;
    const { invoke } = await import("@tauri-apps/api/core");
    await invoke("fs_create_dir", { path });
  }

  async exists(path: string): Promise<boolean> {
    if (!isTauri()) return false;
    const { invoke } = await import("@tauri-apps/api/core");
    return invoke("fs_exists", { path });
  }

  async watch(path: string, callback: (event: FileChangeEvent) => void): Promise<() => void> {
    if (!isTauri()) return () => {};
    const { invoke } = await import("@tauri-apps/api/core");
    const { listen } = await import("@tauri-apps/api/event");

    await invoke("fs_watch", { path });

    const unlisten = await listen<FileChangeEvent>("fs-change", (e) => {
      callback(e.payload);
    });

    return async () => {
      unlisten();
      try {
        await invoke("fs_unwatch");
      } catch {
        // Ignore cleanup errors
      }
    };
  }

  async getDefaultDirs(): Promise<DefaultDirs> {
    if (!isTauri()) {
      return { home: "", documents: "", downloads: "", desktop: "" };
    }
    const { invoke } = await import("@tauri-apps/api/core");
    return invoke("fs_get_default_dirs");
  }

  /** Local-only: get a thumbnail data URL for an image file */
  async getThumbnail(path: string): Promise<string | null> {
    return getCachedThumbnail(path);
  }

  /** Local-only: read a binary file as a base64 data URL */
  async readFileBase64(path: string): Promise<string> {
    if (!isTauri()) return "";
    const { invoke } = await import("@tauri-apps/api/core");
    return invoke("fs_read_file_base64", { path });
  }
}
