import { defineStore } from "pinia";

const STORAGE_KEY = "nitrux-preferences";

export interface Preferences {
  defaultScanDirectory: string;
  dashboardRefreshIntervalMs: number;
  confirmNonDestructiveActions: boolean;
}

const DEFAULTS: Preferences = {
  defaultScanDirectory: "",
  dashboardRefreshIntervalMs: 2000,
  confirmNonDestructiveActions: true,
};

function isPreferences(value: unknown): value is Preferences {
  if (typeof value !== "object" || value === null) return false;
  const v = value as Record<string, unknown>;
  return (
    typeof v.defaultScanDirectory === "string" &&
    typeof v.dashboardRefreshIntervalMs === "number" &&
    typeof v.confirmNonDestructiveActions === "boolean"
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
    setConfirmNonDestructiveActions(value: boolean) {
      this.confirmNonDestructiveActions = value;
      persist({ ...this.$state });
    },
  },
});
