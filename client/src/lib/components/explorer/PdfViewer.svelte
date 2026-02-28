<script lang="ts">
  import ZoomIn from "@lucide/svelte/icons/zoom-in";
  import ZoomOut from "@lucide/svelte/icons/zoom-out";
  import Maximize2 from "@lucide/svelte/icons/maximize-2";
  import ScanLine from "@lucide/svelte/icons/scan-line";
  import ChevronLeft from "@lucide/svelte/icons/chevron-left";
  import ChevronRight from "@lucide/svelte/icons/chevron-right";

  let {
    src,
    filename,
  }: {
    src: string;
    filename: string;
  } = $props();

  let zoom = $state(100);
  let page = $state(1);
  let pageInput = $state("1");

  const ZOOM_STEP = 25;
  const MIN_ZOOM = 25;
  const MAX_ZOOM = 400;

  function zoomIn() {
    zoom = Math.min(MAX_ZOOM, zoom + ZOOM_STEP);
  }

  function zoomOut() {
    zoom = Math.max(MIN_ZOOM, zoom - ZOOM_STEP);
  }

  function fitWidth() {
    zoom = 100;
  }

  function fitPage() {
    zoom = 75;
  }

  function prevPage() {
    if (page > 1) {
      page--;
      pageInput = String(page);
    }
  }

  function nextPage() {
    page++;
    pageInput = String(page);
  }

  function handlePageInput(e: KeyboardEvent) {
    if (e.key === "Enter") {
      const val = parseInt(pageInput);
      if (!isNaN(val) && val >= 1) {
        page = val;
        pageInput = String(page);
      } else {
        pageInput = String(page);
      }
    }
  }

  let embedSrc = $derived(`${src}#page=${page}&zoom=${zoom}`);
</script>

<div class="flex h-full flex-col">
  <!-- Toolbar -->
  <div class="flex items-center gap-1 border-b border-border/50 px-3 py-1.5">
    <!-- Page navigation -->
    <button
      type="button"
      class="rounded-md p-1 text-muted-foreground hover:bg-muted hover:text-foreground"
      onclick={prevPage}
      disabled={page <= 1}
      title="Previous page"
    >
      <ChevronLeft class="h-4 w-4" />
    </button>
    <div class="flex items-center gap-1">
      <input
        type="text"
        bind:value={pageInput}
        onkeydown={handlePageInput}
        class="w-10 rounded border border-border/50 bg-muted/30 px-1.5 py-0.5 text-center text-xs text-foreground"
      />
    </div>
    <button
      type="button"
      class="rounded-md p-1 text-muted-foreground hover:bg-muted hover:text-foreground"
      onclick={nextPage}
      title="Next page"
    >
      <ChevronRight class="h-4 w-4" />
    </button>

    <div class="mx-2 h-4 w-px bg-border/50"></div>

    <!-- Zoom controls -->
    <button
      type="button"
      class="rounded-md p-1 text-muted-foreground hover:bg-muted hover:text-foreground"
      onclick={zoomOut}
      disabled={zoom <= MIN_ZOOM}
      title="Zoom out"
    >
      <ZoomOut class="h-4 w-4" />
    </button>
    <span class="w-10 text-center text-xs tabular-nums text-muted-foreground">{zoom}%</span>
    <button
      type="button"
      class="rounded-md p-1 text-muted-foreground hover:bg-muted hover:text-foreground"
      onclick={zoomIn}
      disabled={zoom >= MAX_ZOOM}
      title="Zoom in"
    >
      <ZoomIn class="h-4 w-4" />
    </button>

    <div class="mx-2 h-4 w-px bg-border/50"></div>

    <!-- Fit options -->
    <button
      type="button"
      class="rounded-md p-1 text-muted-foreground hover:bg-muted hover:text-foreground"
      onclick={fitWidth}
      title="Fit width"
    >
      <ScanLine class="h-4 w-4" />
    </button>
    <button
      type="button"
      class="rounded-md p-1 text-muted-foreground hover:bg-muted hover:text-foreground"
      onclick={fitPage}
      title="Fit page"
    >
      <Maximize2 class="h-4 w-4" />
    </button>
  </div>

  <!-- PDF embed -->
  <div class="flex-1 overflow-hidden">
    <embed
      src={embedSrc}
      type="application/pdf"
      title={filename}
      class="h-full w-full"
    />
  </div>
</div>
