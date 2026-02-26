<script lang="ts">
  import { explorerStore } from "$lib/stores";
  import NavBar from "./NavBar.svelte";
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

      // Ctrl/Cmd+F to focus search
      if ((e.ctrlKey || e.metaKey) && e.key === "f") {
        e.preventDefault();
        window.dispatchEvent(new CustomEvent("explorer:focus-search"));
      }

      // Ctrl/Cmd+A to select all (when not in an input)
      if ((e.ctrlKey || e.metaKey) && e.key === "a" && !explorerStore.isHome && !isInput) {
        e.preventDefault();
        explorerStore.selectAll();
      }

      // F2 to rename selected file
      if (e.key === "F2" && explorerStore.selectedFiles.size === 1 && !isInput) {
        e.preventDefault();
        const selected = [...explorerStore.selectedFiles][0];
        explorerStore.startRename(selected);
      }

      // Delete to delete selected files
      if (e.key === "Delete" && explorerStore.selectedFiles.size > 0 && !isInput) {
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
      }
    }

    document.addEventListener("keydown", handleKeydown);
    return () => document.removeEventListener("keydown", handleKeydown);
  });
</script>

<div class="flex h-full flex-col">
  <NavBar />
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
