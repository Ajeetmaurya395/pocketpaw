<script lang="ts">
  import { explorerStore } from "$lib/stores";
  import { parentDir } from "$lib/filesystem";
  import HomeView from "./HomeView.svelte";
  import FileGrid from "./FileGrid.svelte";
  import FileList from "./FileList.svelte";
  import FileViewer from "./FileViewer.svelte";
  import ChatSidebar from "./ChatSidebar.svelte";
  import { onMount } from "svelte";

  onMount(() => {
    function handleKeydown(e: KeyboardEvent) {
      const tag = (e.target as HTMLElement)?.tagName;
      const isInput = tag === "INPUT" || tag === "TEXTAREA";
      const isEditor = (e.target as HTMLElement)?.closest(".cm-editor") !== null;

      // Skip all custom shortcuts when in input or code editor
      if (isInput || isEditor) return;

      // Ctrl/Cmd+A to select all
      if ((e.ctrlKey || e.metaKey) && e.key === "a" && !explorerStore.isHome) {
        e.preventDefault();
        explorerStore.selectAll();
        return;
      }

      // Ctrl/Cmd+C to copy files
      if ((e.ctrlKey || e.metaKey) && e.key === "c" && explorerStore.selectedFiles.size > 0) {
        e.preventDefault();
        explorerStore.copyToClipboard();
        return;
      }

      // Ctrl/Cmd+X to cut files
      if ((e.ctrlKey || e.metaKey) && e.key === "x" && explorerStore.selectedFiles.size > 0) {
        e.preventDefault();
        explorerStore.cutToClipboard();
        return;
      }

      // Ctrl/Cmd+V to paste files
      if ((e.ctrlKey || e.metaKey) && e.key === "v" && explorerStore.clipboardFiles.size > 0) {
        e.preventDefault();
        explorerStore.paste();
        return;
      }

      // Ctrl+Shift+N to create new folder
      if ((e.ctrlKey || e.metaKey) && e.shiftKey && e.key === "N" && !explorerStore.isHome) {
        e.preventDefault();
        const name = prompt("New folder name:");
        if (name) explorerStore.createFolder(name);
        return;
      }

      // F2 to rename selected file
      if (e.key === "F2" && explorerStore.selectedFiles.size === 1) {
        e.preventDefault();
        const selected = [...explorerStore.selectedFiles][0];
        explorerStore.startRename(selected);
        return;
      }

      // Delete to delete selected files
      if (e.key === "Delete" && explorerStore.selectedFiles.size > 0) {
        e.preventDefault();
        const count = explorerStore.selectedFiles.size;
        if (confirm(`Delete ${count} item${count > 1 ? "s" : ""}?`)) {
          import("$lib/filesystem").then(({ localFs }) => {
            const promises = [...explorerStore.selectedFiles].map((path) => {
              const file = explorerStore.files.find((f) => f.path === path);
              return localFs.deleteFile(path, file?.isDir ?? false);
            });
            Promise.all(promises).then(() => {
              explorerStore.clearSelection();
              explorerStore.refresh();
            });
          });
        }
        return;
      }

      // Arrow keys to navigate files
      if (e.key === "ArrowDown" && !explorerStore.isHome && !explorerStore.isDetailView) {
        e.preventDefault();
        explorerStore.moveFocus(1);
        return;
      }

      if (e.key === "ArrowUp" && !explorerStore.isHome && !explorerStore.isDetailView) {
        e.preventDefault();
        explorerStore.moveFocus(-1);
        return;
      }

      // Enter to open focused file/folder
      if (e.key === "Enter" && explorerStore.focusedIndex >= 0 && !explorerStore.isDetailView) {
        e.preventDefault();
        const focused = explorerStore.sortedFiles[explorerStore.focusedIndex];
        if (focused) {
          if (focused.isDir) {
            explorerStore.navigateTo(focused.path);
          } else {
            explorerStore.openFileDetail(focused);
          }
        }
        return;
      }

      // Backspace to go to parent directory
      if (e.key === "Backspace" && !explorerStore.isHome && !explorerStore.isDetailView) {
        e.preventDefault();
        const parent = parentDir(explorerStore.currentPath);
        if (parent) explorerStore.navigateTo(parent);
        return;
      }

      // Space to quick preview focused file
      if (e.key === " " && explorerStore.focusedIndex >= 0 && !explorerStore.isDetailView) {
        e.preventDefault();
        const focused = explorerStore.sortedFiles[explorerStore.focusedIndex];
        if (focused && !focused.isDir) {
          explorerStore.openFileDetail(focused);
        }
        return;
      }
    }

    document.addEventListener("keydown", handleKeydown);
    return () => document.removeEventListener("keydown", handleKeydown);
  });
</script>

<div class="flex h-full flex-col">
  <div class="flex flex-1 overflow-hidden">
    {#if explorerStore.isDetailView && explorerStore.openFile}
      <div class="flex-1 overflow-hidden">
        <FileViewer file={explorerStore.openFile} />
      </div>
    {:else}
      <div class="flex-1 overflow-y-auto">
        {#if explorerStore.isHome}
          <HomeView />
        {:else if explorerStore.viewMode === "list"}
          <FileList />
        {:else}
          <FileGrid />
        {/if}
      </div>
    {/if}
    {#if explorerStore.chatSidebarOpen}
      <ChatSidebar />
    {/if}
  </div>
</div>
