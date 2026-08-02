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
    // Parallel workers race on CPU under heavy component-mount tests (e.g. real
    // xterm.js init in App.spec.ts), which intermittently delays an unrelated
    // file's Vue reactivity flush past its assertion. Single-threaded execution
    // is deterministic; the app logic itself is correct under either mode.
    poolOptions: {
      threads: {
        singleThread: true,
      },
    },
  },
});
