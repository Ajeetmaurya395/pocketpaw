<script lang="ts">
  import { page } from "$app/state";
  import { MessageSquare, FolderOpen, Rocket, FolderKanban } from "@lucide/svelte";
  import type { Component } from "svelte";

  const tabs: { href: string; label: string; icon: Component<any>; match: (p: string) => boolean }[] = [
    { href: "/chat", label: "Chat", icon: MessageSquare, match: (p) => p.startsWith("/chat") },
    { href: "/", label: "Files", icon: FolderOpen, match: (p) => p === "/" },
    { href: "/command-center", label: "PawKits", icon: Rocket, match: (p) => p.startsWith("/command-center") },
    { href: "/projects", label: "Deep Work", icon: FolderKanban, match: (p) => p.startsWith("/projects") },
  ];

  let pathname = $derived(page.url.pathname);
</script>

<nav class="flex items-center gap-0.5">
  {#each tabs as tab (tab.href)}
    {@const active = tab.match(pathname)}
    {@const Icon = tab.icon}
    <a
      href={tab.href}
      data-sveltekit-noscroll
      class={[
        "flex items-center gap-1.5 rounded-md px-2.5 py-1 text-[12px] font-medium transition-colors duration-100",
        active
          ? "bg-foreground/10 text-foreground"
          : "text-muted-foreground hover:bg-foreground/5 hover:text-foreground/80",
      ].join(" ")}
    >
      <Icon class="h-3 w-3" strokeWidth={2} />
      {tab.label}
    </a>
  {/each}
</nav>
