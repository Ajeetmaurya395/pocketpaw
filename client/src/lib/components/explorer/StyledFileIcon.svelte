<script lang="ts">
  let {
    extension = "",
    isDir = false,
    size = 36,
  }: {
    extension?: string;
    isDir?: boolean;
    size?: number;
  } = $props();

  // Category color palette
  const CATEGORY_COLORS: Record<string, { fill: string; text: string }> = {
    // Code
    py: { fill: "#3572A5", text: "PY" },
    pyw: { fill: "#3572A5", text: "PY" },
    ts: { fill: "#3178C6", text: "TS" },
    tsx: { fill: "#3178C6", text: "TSX" },
    js: { fill: "#F7DF1E", text: "JS" },
    jsx: { fill: "#F7DF1E", text: "JSX" },
    rs: { fill: "#DEA584", text: "RS" },
    go: { fill: "#00ADD8", text: "GO" },
    c: { fill: "#555555", text: "C" },
    cpp: { fill: "#004482", text: "C++" },
    h: { fill: "#004482", text: "H" },
    java: { fill: "#B07219", text: "JAVA" },
    kt: { fill: "#A97BFF", text: "KT" },
    swift: { fill: "#F05138", text: "SWIFT" },
    rb: { fill: "#CC342D", text: "RB" },
    php: { fill: "#4F5D95", text: "PHP" },
    cs: { fill: "#68217A", text: "C#" },
    dart: { fill: "#00B4AB", text: "DART" },
    svelte: { fill: "#FF3E00", text: "SVLT" },
    vue: { fill: "#42B883", text: "VUE" },
    html: { fill: "#E34F26", text: "HTML" },
    css: { fill: "#1572B6", text: "CSS" },
    scss: { fill: "#CC6699", text: "SCSS" },
    less: { fill: "#1D365D", text: "LESS" },
    sh: { fill: "#4EAA25", text: "SH" },
    bash: { fill: "#4EAA25", text: "SH" },
    zsh: { fill: "#4EAA25", text: "ZSH" },
    sql: { fill: "#E38C00", text: "SQL" },

    // Documents
    pdf: { fill: "#E53E3E", text: "PDF" },
    doc: { fill: "#2B579A", text: "DOC" },
    docx: { fill: "#2B579A", text: "DOCX" },
    rtf: { fill: "#2B579A", text: "RTF" },
    odt: { fill: "#2B579A", text: "ODT" },
    md: { fill: "#6B7280", text: "MD" },
    txt: { fill: "#6B7280", text: "TXT" },
    log: { fill: "#6B7280", text: "LOG" },

    // Spreadsheets
    xlsx: { fill: "#217346", text: "XLSX" },
    xls: { fill: "#217346", text: "XLS" },
    csv: { fill: "#217346", text: "CSV" },
    ods: { fill: "#217346", text: "ODS" },

    // Presentations
    pptx: { fill: "#B7472A", text: "PPTX" },
    ppt: { fill: "#B7472A", text: "PPT" },
    odp: { fill: "#B7472A", text: "ODP" },

    // Images
    png: { fill: "#9333EA", text: "PNG" },
    jpg: { fill: "#9333EA", text: "JPG" },
    jpeg: { fill: "#9333EA", text: "JPEG" },
    gif: { fill: "#9333EA", text: "GIF" },
    svg: { fill: "#9333EA", text: "SVG" },
    webp: { fill: "#9333EA", text: "WEBP" },
    bmp: { fill: "#9333EA", text: "BMP" },
    ico: { fill: "#9333EA", text: "ICO" },
    tiff: { fill: "#9333EA", text: "TIFF" },

    // Video
    mp4: { fill: "#EA580C", text: "MP4" },
    mov: { fill: "#EA580C", text: "MOV" },
    avi: { fill: "#EA580C", text: "AVI" },
    webm: { fill: "#EA580C", text: "WEBM" },
    mkv: { fill: "#EA580C", text: "MKV" },
    flv: { fill: "#EA580C", text: "FLV" },

    // Audio
    mp3: { fill: "#EC4899", text: "MP3" },
    wav: { fill: "#EC4899", text: "WAV" },
    flac: { fill: "#EC4899", text: "FLAC" },
    ogg: { fill: "#EC4899", text: "OGG" },
    aac: { fill: "#EC4899", text: "AAC" },
    m4a: { fill: "#EC4899", text: "M4A" },

    // Archives
    zip: { fill: "#B45309", text: "ZIP" },
    tar: { fill: "#B45309", text: "TAR" },
    gz: { fill: "#B45309", text: "GZ" },
    rar: { fill: "#B45309", text: "RAR" },
    "7z": { fill: "#B45309", text: "7Z" },
    bz2: { fill: "#B45309", text: "BZ2" },

    // Data / Config
    json: { fill: "#CBB133", text: "JSON" },
    yaml: { fill: "#6B8E23", text: "YAML" },
    yml: { fill: "#6B8E23", text: "YML" },
    toml: { fill: "#9CA3AF", text: "TOML" },
    xml: { fill: "#9CA3AF", text: "XML" },
    ini: { fill: "#9CA3AF", text: "INI" },
    env: { fill: "#9CA3AF", text: "ENV" },
    lock: { fill: "#9CA3AF", text: "LOCK" },

    // Executables / binaries
    exe: { fill: "#374151", text: "EXE" },
    dll: { fill: "#374151", text: "DLL" },
    so: { fill: "#374151", text: "SO" },
    dmg: { fill: "#374151", text: "DMG" },
    msi: { fill: "#374151", text: "MSI" },
    deb: { fill: "#374151", text: "DEB" },
    rpm: { fill: "#374151", text: "RPM" },
    wasm: { fill: "#654FF0", text: "WASM" },
  };

  const DEFAULT_COLOR = { fill: "#6B7280", text: "" };

  let resolved = $derived.by(() => {
    const ext = extension.toLowerCase();
    const cat = CATEGORY_COLORS[ext];
    if (cat) return cat;
    // If we have an extension but no mapping, show it as label with generic color
    if (ext) return { fill: DEFAULT_COLOR.fill, text: ext.toUpperCase().slice(0, 4) };
    return DEFAULT_COLOR;
  });

  // Scale factor relative to default 36px
  let scale = $derived(size / 36);
</script>

{#if isDir}
  <!-- Folder icon: filled folder shape with tab -->
  <svg
    width={size}
    height={size}
    viewBox="0 0 36 36"
    fill="none"
    xmlns="http://www.w3.org/2000/svg"
  >
    <defs>
      <linearGradient id="folderGrad" x1="0" y1="0" x2="0" y2="1">
        <stop offset="0%" stop-color="#FBBF24" />
        <stop offset="100%" stop-color="#D97706" />
      </linearGradient>
    </defs>
    <!-- Folder tab -->
    <path
      d="M4 10 L4 8 Q4 6 6 6 L14 6 Q15 6 15.5 7 L17 10 Z"
      fill="url(#folderGrad)"
    />
    <!-- Folder body -->
    <rect x="3" y="10" width="30" height="20" rx="3" fill="url(#folderGrad)" />
    <!-- Highlight line -->
    <rect x="5" y="12" width="26" height="1.5" rx="0.75" fill="#FDE68A" opacity="0.5" />
  </svg>
{:else}
  <!-- File icon: rounded rect with dog-ear fold + extension label -->
  <svg
    width={size}
    height={size}
    viewBox="0 0 36 36"
    fill="none"
    xmlns="http://www.w3.org/2000/svg"
  >
    <!-- File body with dog-ear -->
    <path
      d="M6 4 L22 4 L30 12 L30 32 Q30 34 28 34 L8 34 Q6 34 6 32 Z"
      fill={resolved.fill}
      opacity="0.9"
    />
    <!-- Dog-ear fold -->
    <path
      d="M22 4 L22 10 Q22 12 24 12 L30 12 Z"
      fill="white"
      opacity="0.3"
    />
    <!-- Extension label -->
    {#if resolved.text}
      <text
        x="18"
        y="27"
        text-anchor="middle"
        font-size={resolved.text.length > 3 ? "7" : "8.5"}
        font-weight="700"
        font-family="Inter, system-ui, sans-serif"
        fill="white"
        opacity="0.95"
      >
        {resolved.text}
      </text>
    {/if}
  </svg>
{/if}
