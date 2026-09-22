#!/usr/bin/env node
// Fails when the test run did not execute every spec file.
//
// Vitest exits 0 when a worker fails to *start*: the files it was meant to
// run are simply never executed, reported as "Errors" rather than
// failures. On this project that is not hypothetical -- from WSL2 against
// /mnt/d, a full run once executed 14 of 77 files and still exited 0.
// A green run that skipped four files in five is worse than a red one, so
// this compares what ran against what exists on disk.

import { readFileSync, readdirSync, statSync } from "node:fs";
import { join, relative } from "node:path";

const ROOT = new URL("..", import.meta.url).pathname;
const REPORT = join(ROOT, ".vitest-report.json");

function findSpecFiles(dir) {
  const found = [];
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    if (entry.name === "node_modules" || entry.name.startsWith(".")) continue;
    const full = join(dir, entry.name);
    if (entry.isDirectory()) {
      found.push(...findSpecFiles(full));
    } else if (/\.spec\.[tj]s$/.test(entry.name)) {
      found.push(relative(ROOT, full).replace(/\\/g, "/"));
    }
  }
  return found;
}

let report;
try {
  statSync(REPORT);
  report = JSON.parse(readFileSync(REPORT, "utf8"));
} catch {
  console.error(`assert-suite-complete: rapport introuvable (${REPORT}).`);
  process.exit(1);
}

const executed = new Set(
  (report.testResults ?? []).map((r) => relative(ROOT, r.name).replace(/\\/g, "/")),
);
const expected = findSpecFiles(join(ROOT, "src"));
const missing = expected.filter((f) => !executed.has(f));

if (missing.length > 0) {
  console.error(
    `assert-suite-complete: ${missing.length} fichier(s) de test sur ${expected.length} n'ont pas été exécutés :`,
  );
  for (const file of missing) console.error(`  - ${file}`);
  console.error(
    "Cause habituelle sur cette machine : un worker vitest n'a pas démarré dans le délai imparti (WSL2 sur /mnt/d).",
  );
  console.error("Relancer ces fichiers seuls pour les vérifier, ou relancer la suite.");
  process.exit(1);
}

console.log(`assert-suite-complete: ${expected.length}/${expected.length} fichiers de test exécutés.`);
