import { defineStore } from "pinia";
import type { Theme } from "@/types/theme";
import { builtinThemes } from "@/themes/builtin";

const CSS_VAR_MAP: Record<keyof Theme["colors"], string> = {
  bgBase: "--nx-bg-base",
  bgElevated: "--nx-bg-elevated",
  bgOverlay: "--nx-bg-overlay",
  border: "--nx-border",
  textPrimary: "--nx-text-primary",
  textSecondary: "--nx-text-secondary",
  accentPrimary: "--nx-accent-primary",
  accentSecondary: "--nx-accent-secondary",
  accentSuccess: "--nx-accent-success",
  accentWarning: "--nx-accent-warning",
  accentDanger: "--nx-accent-danger",
};

// Derived from CSS_VAR_MAP (which is exhaustively typed over Theme["colors"])
// so a new color key can never drift out of sync with the validation list.
export const REQUIRED_COLOR_KEYS = Object.keys(CSS_VAR_MAP) as (keyof Theme["colors"])[];

function hasAllRequiredColors(colors: unknown): colors is Theme["colors"] {
  if (typeof colors !== "object" || colors === null) return false;
  return REQUIRED_COLOR_KEYS.every((key) => key in (colors as Record<string, unknown>));
}

function applyToDom(theme: Theme) {
  const root = document.documentElement;
  for (const [key, cssVar] of Object.entries(CSS_VAR_MAP)) {
    root.style.setProperty(cssVar, theme.colors[key as keyof Theme["colors"]]);
  }
  root.dataset.themeMode = theme.mode;
}

const STORAGE_KEY = "nitrux-theme";
const DEFAULT_THEME = builtinThemes.find((t) => t.id === "oled-noir") as Theme;

function readPersistedTheme(): Theme {
  const stored = localStorage.getItem(STORAGE_KEY);
  if (!stored) return DEFAULT_THEME;
  try {
    const parsed = JSON.parse(stored) as Theme;
    if (parsed?.id && hasAllRequiredColors(parsed.colors)) return parsed;
  } catch {
    /* fall through to default */
  }
  return DEFAULT_THEME;
}

export const useThemeStore = defineStore("theme", {
  state: () => ({
    active: readPersistedTheme(),
    customThemes: [] as Theme[],
  }),
  actions: {
    setTheme(theme: Theme) {
      this.active = theme;
      applyToDom(theme);
      localStorage.setItem(STORAGE_KEY, JSON.stringify(theme));
    },
    updateActiveColor(key: keyof Theme["colors"], value: string) {
      this.active = { ...this.active, colors: { ...this.active.colors, [key]: value } };
      applyToDom(this.active);
    },
    saveCustomTheme(theme: Theme) {
      const existingIndex = this.customThemes.findIndex((t) => t.id === theme.id);
      if (existingIndex >= 0) this.customThemes.splice(existingIndex, 1, theme);
      else this.customThemes.push(theme);
    },
    exportActiveTheme(): string {
      return JSON.stringify(this.active, null, 2);
    },
    importTheme(json: string): { ok: true } | { ok: false; error: string } {
      try {
        const parsed = JSON.parse(json) as Theme;
        if (!parsed.id || !parsed.colors) {
          return { ok: false, error: "Fichier de thème invalide : id ou colors manquant." };
        }
        if (!hasAllRequiredColors(parsed.colors)) {
          return { ok: false, error: "Fichier de thème invalide : couleurs manquantes dans colors." };
        }
        this.saveCustomTheme(parsed);
        return { ok: true };
      } catch {
        return { ok: false, error: "JSON invalide." };
      }
    },
  },
});
