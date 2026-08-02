<!-- src/pages/TerminalPage.vue -->
<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import { invoke, Channel } from "@tauri-apps/api/core";
import { Terminal } from "@xterm/xterm";
import { FitAddon } from "@xterm/addon-fit";
import NxSectionHeader from "@/components/ui/NxSectionHeader.vue";
import "@xterm/xterm/css/xterm.css";

const containerEl = ref<HTMLDivElement | null>(null);
const id = crypto.randomUUID();

let term: Terminal | null = null;
let fitAddon: FitAddon | null = null;
let resizeObserver: ResizeObserver | null = null;

async function resizeAndNotify() {
  if (!term || !fitAddon) return;
  fitAddon.fit();
  await invoke("resize_terminal", { id, rows: term.rows, cols: term.cols });
}

onMounted(async () => {
  term = new Terminal({ cursorBlink: true, fontSize: 13 });
  fitAddon = new FitAddon();
  term.loadAddon(fitAddon);
  if (containerEl.value) {
    term.open(containerEl.value);
  }

  term.onData((data: string) => {
    invoke("write_to_terminal", { id, data });
  });

  const onData = new Channel<string>();
  onData.onmessage = (data: string) => {
    term?.write(data);
  };
  await invoke("spawn_terminal", { id, onData });

  await resizeAndNotify();
  // ResizeObserver is a standard Web API present in the real Tauri webview
  // (WebKitGTK) but absent from jsdom's test environment -- guarded so
  // the component test suite doesn't need a polyfill for something that
  // works fine in the actual app.
  if (containerEl.value && typeof ResizeObserver !== "undefined") {
    resizeObserver = new ResizeObserver(() => resizeAndNotify());
    resizeObserver.observe(containerEl.value);
  }
});

onUnmounted(() => {
  resizeObserver?.disconnect();
  invoke("close_terminal", { id });
  term?.dispose();
});
</script>

<template>
  <div class="term-page">
    <NxSectionHeader title="Terminal" description="Shell interactif, mêmes droits que votre session." />
    <div ref="containerEl" class="term-container"></div>
  </div>
</template>

<style scoped>
.term-page { padding: 24px; display: flex; flex-direction: column; gap: 12px; height: 100%; box-sizing: border-box; }
.term-container { flex: 1; min-height: 0; background: #000; border-radius: 8px; padding: 8px; }
</style>
