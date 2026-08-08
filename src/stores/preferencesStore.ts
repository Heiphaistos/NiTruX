import { defineStore } from "pinia";

const STORAGE_KEY = "nitrux-preferences";

export interface Preferences {
  defaultScanDirectory: string;
  dashboardRefreshIntervalMs: number;
  cpuAlertThreshold: number;
  ramAlertThreshold: number;
  diskAlertThreshold: number;
}

const DEFAULTS: Preferences = {
  defaultScanDirectory: "",
  dashboardRefreshIntervalMs: 2000,
  cpuAlertThreshold: 80,
  ramAlertThreshold: 80,
  diskAlertThreshold: 85,
};

// The <input type="number" min="1" max="100"> attributes on the settings
// form are purely decorative -- v-model.number never enforces them, so a
// field cleared by a stray backspace (or just typed "0") would otherwise
// persist as 0, and every usage check against it (>= 0%) would then read
// as "always exceeded" forever, even at genuinely 0% load. This exact bug
// class bit NiTriTe's own Windows sibling twice (AlertThresholdsModal.vue,
// then StatsReportsPage.vue) before it clamped on save instead of trusting
// the HTML attributes -- applying that lesson here from the start rather
// than rediscovering it.
function clampThreshold(n: number, fallback: number): number {
  if (!Number.isFinite(n)) return fallback;
  return Math.min(100, Math.max(1, Math.round(n)));
}

function isPreferences(value: unknown): value is Preferences {
  if (typeof value !== "object" || value === null) return false;
  const v = value as Record<string, unknown>;
  return (
    typeof v.defaultScanDirectory === "string" &&
    typeof v.dashboardRefreshIntervalMs === "number" &&
    typeof v.cpuAlertThreshold === "number" &&
    typeof v.ramAlertThreshold === "number" &&
    typeof v.diskAlertThreshold === "number"
  );
}

function readPersistedPreferences(): Preferences {
  const stored = localStorage.getItem(STORAGE_KEY);
  if (!stored) return { ...DEFAULTS };
  try {
    const parsed: unknown = JSON.parse(stored);
    return isPreferences(parsed) ? parsed : { ...DEFAULTS };
  } catch {
    return { ...DEFAULTS };
  }
}

function persist(prefs: Preferences) {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(prefs));
}

export const usePreferencesStore = defineStore("preferences", {
  state: (): Preferences => readPersistedPreferences(),
  actions: {
    setDefaultScanDirectory(value: string) {
      this.defaultScanDirectory = value;
      persist({ ...this.$state });
    },
    setDashboardRefreshIntervalMs(value: number) {
      this.dashboardRefreshIntervalMs = value;
      persist({ ...this.$state });
    },
    setCpuAlertThreshold(value: number) {
      this.cpuAlertThreshold = clampThreshold(value, DEFAULTS.cpuAlertThreshold);
      persist({ ...this.$state });
    },
    setRamAlertThreshold(value: number) {
      this.ramAlertThreshold = clampThreshold(value, DEFAULTS.ramAlertThreshold);
      persist({ ...this.$state });
    },
    setDiskAlertThreshold(value: number) {
      this.diskAlertThreshold = clampThreshold(value, DEFAULTS.diskAlertThreshold);
      persist({ ...this.$state });
    },
  },
});
