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
    // out to every core. Capped at 2 workers the full suite runs, though
    // not with certainty -- one run in three still drops a file or two
    // under load. The startup budget itself is not configurable: it is a
    // hardcoded `START_TIMEOUT = 6e4` in vitest's own pool runner, with no
    // option or environment variable behind it. `npm test` therefore also
    // runs scripts/assert-suite-complete.mjs, which fails the run when a
    // spec file never executed -- never trust the failure count alone.
    pool: "threads",
    maxWorkers: 2,
  },
});
