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
const CUSTOM_THEMES_STORAGE_KEY = "nitrux-custom-themes";
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

// Mirrors readPersistedTheme's defensive parsing -- a saved/imported custom
// theme is otherwise silently lost from the swatch picker on every reload
// (only the currently *active* theme was ever persisted; the list of saved
// custom themes itself never was).
function readPersistedCustomThemes(): Theme[] {
  const stored = localStorage.getItem(CUSTOM_THEMES_STORAGE_KEY);
  if (!stored) return [];
  try {
    const parsed = JSON.parse(stored) as Theme[];
    if (!Array.isArray(parsed)) return [];
    return parsed.filter((t) => t?.id && hasAllRequiredColors(t.colors));
  } catch {
    return [];
  }
}

export const useThemeStore = defineStore("theme", {
  state: () => ({
    active: readPersistedTheme(),
    customThemes: readPersistedCustomThemes(),
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
      localStorage.setItem(CUSTOM_THEMES_STORAGE_KEY, JSON.stringify(this.customThemes));
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
        this.setTheme(parsed);
        return { ok: true };
      } catch {
        return { ok: false, error: "JSON invalide." };
      }
    },
  },
});
