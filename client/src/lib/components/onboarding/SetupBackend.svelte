<script lang="ts">
  import { Loader2, Download, Play, AlertCircle, ExternalLink, Box, Sparkles, Layers } from "@lucide/svelte";
  import { isTauri } from "$lib/auth";

  type BackendState = "backend_missing" | "backend_stopped" | "installing" | "starting";

  let {
    backendState: initialState,
    onReady,
  }: {
    backendState: BackendState;
    onReady: () => void;
  } = $props();

  let currentState = $state<BackendState>(initialState);
  let installLogs = $state<string[]>([]);
  let error = $state<string | null>(null);
  let logContainer: HTMLDivElement | undefined = $state(undefined);
  let unlistenInstall: (() => void) | null = null;
  let selectedProfile = $state<"minimal" | "recommended" | "full">("recommended");

  const profiles = [
    { id: "minimal" as const, label: "Minimal", desc: "Core agent only, no extras", icon: Box },
    { id: "recommended" as const, label: "Recommended", desc: "Dashboard, browser, channels", icon: Sparkles },
    { id: "full" as const, label: "Full", desc: "Everything including experimental", icon: Layers },
  ];

  // Sync prop changes into internal state
  $effect(() => {
    currentState = initialState;
  });

  // Auto-scroll logs
  $effect(() => {
    if (logContainer && installLogs.length) {
      logContainer.scrollTop = logContainer.scrollHeight;
    }
  });

  async function startInstall() {
    if (!isTauri()) return;
    currentState = "installing";
    installLogs = [];
    error = null;

    try {
      const { listen } = await import("@tauri-apps/api/event");
      const { invoke } = await import("@tauri-apps/api/core");

      unlistenInstall?.();
      unlistenInstall = await listen<{ line: string; done: boolean; success: boolean }>(
        "install-progress",
        (event) => {
          installLogs = [...installLogs, event.payload.line];
          if (event.payload.done) {
            unlistenInstall?.();
            unlistenInstall = null;
            if (event.payload.success) {
              startBackend();
            } else {
              error = "Installation failed. Check the log above for details.";
              currentState = "backend_missing";
            }
          }
        },
      );

      await invoke("install_pocketpaw", { profile: selectedProfile });
    } catch (e: any) {
      error = e?.message ?? "Failed to start installer.";
      currentState = "backend_missing";
    }
  }

  async function startBackend() {
    if (!isTauri()) return;
    currentState = "starting";
    error = null;

    try {
      const { invoke } = await import("@tauri-apps/api/core");
      await invoke("start_pocketpaw_backend", { port: 8888 });

      // Poll check_backend_running every 1s for up to 30s
      let attempts = 0;
      const poll = setInterval(async () => {
        attempts++;
        try {
          const running = await invoke<boolean>("check_backend_running", { port: 8888 });
          if (running) {
            // Verify it's actually PocketPaw
            try {
              const resp = await fetch("http://localhost:8888/api/v1/version");
              const data = await resp.json();
              if (data?.version) {
                clearInterval(poll);
                onReady();
                return;
              }
            } catch {
              // Port open but not PocketPaw yet — keep polling
            }
          } else if (attempts >= 30) {
            clearInterval(poll);
            error = "Backend did not start within 30 seconds.";
            currentState = "backend_stopped";
          }
        } catch {
          if (attempts >= 30) {
            clearInterval(poll);
            error = "Could not verify backend status.";
            currentState = "backend_stopped";
          }
        }
      }, 1000);
    } catch (e: any) {
      error = e?.message ?? "Failed to start backend.";
      currentState = "backend_stopped";
    }
  }

  function retry() {
    error = null;
    if (currentState === "backend_missing") {
      startInstall();
    } else {
      startBackend();
    }
  }
</script>

<div class="flex h-full w-full items-center justify-center">
  <div class="flex w-full max-w-md flex-col items-center gap-6 px-6 text-center">
    <span class="text-5xl">🐾</span>

    {#if currentState === "backend_missing"}
      <div class="flex flex-col gap-2">
        <h1 class="text-2xl font-semibold text-foreground">PocketPaw Not Installed</h1>
        <p class="text-sm text-muted-foreground">
          Choose what to install, then hit the button below.
        </p>
      </div>

      <!-- Profile selector -->
      <div class="flex w-full flex-col gap-2">
        {#each profiles as p}
          {@const Icon = p.icon}
          <button
            onclick={() => (selectedProfile = p.id)}
            class="flex items-center gap-3 rounded-xl border-2 p-3.5 text-left transition-all
              {selectedProfile === p.id
                ? 'border-primary bg-primary/5'
                : 'border-border hover:border-primary/30'}"
          >
            <div class="flex h-9 w-9 shrink-0 items-center justify-center rounded-lg {selectedProfile === p.id ? 'bg-primary/15' : 'bg-muted'}">
              <Icon class="h-4 w-4 {selectedProfile === p.id ? 'text-primary' : 'text-muted-foreground'}" />
            </div>
            <div class="flex flex-col">
              <span class="text-sm font-medium text-foreground">{p.label}</span>
              <span class="text-xs text-muted-foreground">{p.desc}</span>
            </div>
          </button>
        {/each}
      </div>

      <button
        onclick={startInstall}
        class="inline-flex items-center gap-2 rounded-lg bg-primary px-6 py-2.5 text-sm font-medium text-primary-foreground transition-opacity hover:opacity-90"
      >
        <Download class="h-4 w-4" />
        Install PocketPaw
      </button>

      <a
        href="https://github.com/pocketpaw/pocketpaw#installation"
        target="_blank"
        rel="noopener noreferrer"
        class="inline-flex items-center gap-1 text-xs text-muted-foreground transition-colors hover:text-foreground"
      >
        Manual installation guide
        <ExternalLink class="h-3 w-3" />
      </a>

    {:else if currentState === "installing"}
      <div class="flex flex-col gap-2">
        <h1 class="text-2xl font-semibold text-foreground">Installing PocketPaw</h1>
        <p class="text-sm text-muted-foreground">This may take a minute...</p>
      </div>

      <div class="flex items-center gap-2">
        <Loader2 class="h-4 w-4 animate-spin text-primary" />
        <span class="text-sm text-muted-foreground">Installing...</span>
      </div>

      {#if installLogs.length > 0}
        <div
          bind:this={logContainer}
          class="max-h-48 w-full overflow-y-auto rounded-lg border border-border bg-muted/50 p-3 text-left font-mono text-xs text-muted-foreground"
        >
          {#each installLogs as line}
            <div>{line}</div>
          {/each}
        </div>
      {/if}

    {:else if currentState === "backend_stopped"}
      <div class="flex flex-col gap-2">
        <h1 class="text-2xl font-semibold text-foreground">Backend Not Running</h1>
        <p class="text-sm text-muted-foreground">
          PocketPaw is installed but the backend server isn't running.
        </p>
      </div>

      <button
        onclick={startBackend}
        class="inline-flex items-center gap-2 rounded-lg bg-primary px-6 py-2.5 text-sm font-medium text-primary-foreground transition-opacity hover:opacity-90"
      >
        <Play class="h-4 w-4" />
        Start Backend
      </button>

    {:else if currentState === "starting"}
      <div class="flex flex-col gap-2">
        <h1 class="text-2xl font-semibold text-foreground">Starting Backend</h1>
        <p class="text-sm text-muted-foreground">Waiting for the server to come online...</p>
      </div>

      <div class="flex items-center gap-2">
        <Loader2 class="h-4 w-4 animate-spin text-primary" />
        <span class="text-sm text-muted-foreground">Starting...</span>
      </div>
    {/if}

    {#if error}
      <div class="flex w-full items-start gap-2 rounded-lg border border-destructive/30 bg-destructive/10 p-3 text-left">
        <AlertCircle class="mt-0.5 h-4 w-4 shrink-0 text-destructive" />
        <div class="flex flex-col gap-2">
          <p class="text-xs text-destructive">{error}</p>
          <button
            onclick={retry}
            class="self-start text-xs font-medium text-primary transition-opacity hover:opacity-80"
          >
            Try Again
          </button>
        </div>
      </div>
    {/if}
  </div>
</div>
