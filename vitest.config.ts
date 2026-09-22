import { defineConfig } from "vitest/config";
import vue from "@vitejs/plugin-vue";

export default defineConfig({
  plugins: [vue()],
  resolve: {
    alias: {
      "@": "/src",
    },
  },
  test: {
    environment: "jsdom",
    globals: true,
    exclude: ["**/node_modules/**", "**/.git/**", "**/.worktrees/**"],
    // Vitest 4 defaults to the `forks` pool: one OS process per test file.
    // This repo is developed from WSL2 against /mnt/d (DrvFs), where a
    // process spawn is slow enough that workers miss the pool's startup
    // timeout under load -- measured on the full suite: 63 "Failed to start
    // forks worker / Timeout waiting for worker to respond" errors and only
    // 14 of 77 files executed, with zero real failures among them. The
    // silent part is what makes it dangerous: the run still exits 0.
    // Worker threads boot inside the existing process and do not hit this
    // as hard, but `threads` alone still lost 20 of 77 files when it fanned
    // out to every core. Capped at 2 workers it is reliable: 77/77 files,
    // 511/511 tests, ~6 min. Raise the cap only with a full run to prove
    // it, and check the file count, not just the failure count -- a run
    // that never started a file reports no failures.
    pool: "threads",
    maxWorkers: 2,
  },
});
