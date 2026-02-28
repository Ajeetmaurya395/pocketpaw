<script lang="ts">
  import { explorerStore } from "$lib/stores";
  import ChatPanel from "$lib/components/chat/ChatPanel.svelte";
  import X from "@lucide/svelte/icons/x";
</script>

<div class="flex h-full w-[360px] shrink-0 flex-col border-l border-border ">
  <!-- Header -->
  <div class="flex h-11 items-center justify-between border-b border-border px-3">
    <span class="text-sm font-medium text-foreground">Chat</span>
    <button
      type="button"
      class="rounded-md p-1 text-muted-foreground transition-colors hover:bg-muted hover:text-foreground"
      onclick={() => explorerStore.toggleChatSidebar()}
      title="Close chat"
    >
      <X class="h-4 w-4" />
    </button>
  </div>

  <!-- Chat body -->
  <div class="flex-1 overflow-hidden">
    <ChatPanel />
  </div>
</div>
