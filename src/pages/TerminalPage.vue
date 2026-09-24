<!-- src/pages/TerminalPage.vue -->
<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import { invoke, Channel } from "@tauri-apps/api/core";
import { Terminal } from "@xterm/xterm";
import { FitAddon } from "@xterm/addon-fit";
import NxCard from "@/components/ui/NxCard.vue";
import NxSectionHeader from "@/components/ui/NxSectionHeader.vue";
import NxButton from "@/components/ui/NxButton.vue";
import "@xterm/xterm/css/xterm.css";

const containerEl = ref<HTMLDivElement | null>(null);
// A fresh id per shell session: after `exit`, "Nouvelle session" spawns a
// new backend session under a new id rather than reusing the dead one.
let id = crypto.randomUUID();
const spawnError = ref<string | null>(null);
const exited = ref(false);

let term: Terminal | null = null;
let fitAddon: FitAddon | null = null;
let resizeObserver: ResizeObserver | null = null;

async function resizeAndNotify() {
  if (!term || !fitAddon) return;
  fitAddon.fit();
  // .catch, not try/catch: unlike spawn_terminal, there is nothing more the
  // UI can do about a failed resize (no separate error state to show) --
  // this only guards against an unhandled promise rejection reaching the
  // console when the pty is already gone (e.g. right as the tab closes).
  await invoke("resize_terminal", { id, rows: term.rows, cols: term.cols }).catch(() => {});
}

async function startSession() {
  const onData = new Channel<string>();
  onData.onmessage = (data: string) => {
    term?.write(data);
  };
  // Without this the page could not tell a finished shell (`exit`, Ctrl-D)
  // from an idle one: keystrokes went nowhere and nothing said why.
  const onExit = new Channel<null>();
  const sessionId = id;
  onExit.onmessage = () => {
    if (sessionId !== id) return;
    exited.value = true;
    term?.write("\r\n\x1b[90m[session terminée]\x1b[0m\r\n");
  };
  // Unlike every other page's invoke() call, spawn_terminal failing here
  // (pty open error, $SHELL missing/invalid) previously had no try/catch
  // and no visible error: the user would just see a permanently empty
  // black box with no indication anything went wrong.
  try {
    await invoke("spawn_terminal", { id, onData, onExit });
  } catch (e) {
    spawnError.value = String(e);
    return false;
  }
  exited.value = false;
  spawnError.value = null;
  return true;
}

async function restart() {
  invoke("close_terminal", { id }).catch(() => {});
  id = crypto.randomUUID();
  term?.reset();
  if (await startSession()) {
    await resizeAndNotify();
    term?.focus();
  }
}

onMounted(async () => {
  term = new Terminal({ cursorBlink: true, fontSize: 13 });
  fitAddon = new FitAddon();
  term.loadAddon(fitAddon);
  if (containerEl.value) {
    term.open(containerEl.value);
  }

  term.onData((data: string) => {
    // .catch, not try/catch: fires on every keystroke, so a dead pty (once
    // spawn_terminal has already succeeded) would otherwise spam an
    // unhandled promise rejection per character typed instead of just
    // once. A finished shell is surfaced by the on_exit channel instead.
    if (exited.value) return;
    invoke("write_to_terminal", { id, data }).catch(() => {});
  });

  // The resize observer is wired up even if this first spawn fails, so a
  // successful "Nouvelle session" afterwards still tracks the window size.
  if (await startSession()) await resizeAndNotify();
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
  invoke("close_terminal", { id }).catch(() => {});
  term?.dispose();
});
</script>

<template>
  <div class="term-page">
    <NxSectionHeader title="Terminal" description="Shell interactif, mêmes droits que votre session." />
    <NxCard v-if="spawnError" danger>{{ spawnError }}</NxCard>
    <div v-if="exited || spawnError" class="term-actions">
      <NxButton @click="restart">Nouvelle session</NxButton>
    </div>
    <div v-show="!spawnError" ref="containerEl" class="term-container"></div>
  </div>
</template>

<style scoped>
.term-page { padding: 24px; display: flex; flex-direction: column; gap: 12px; height: 100%; box-sizing: border-box; }
.term-actions { display: flex; gap: 8px; }
.term-container { flex: 1; min-height: 0; background: #000; border-radius: 8px; padding: 8px; }
</style>
