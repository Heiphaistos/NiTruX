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

function applyToDom(theme: Theme) {
  const root = document.documentElement;
  for (const [key, cssVar] of Object.entries(CSS_VAR_MAP)) {
    root.style.setProperty(cssVar, theme.colors[key as keyof Theme["colors"]]);
  }
  root.dataset.themeMode = theme.mode;
}

export const useThemeStore = defineStore("theme", {
  state: () => ({
    active: builtinThemes[0] as Theme,
    customThemes: [] as Theme[],
  }),
  actions: {
    setTheme(theme: Theme) {
      this.active = theme;
      applyToDom(theme);
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
        this.saveCustomTheme(parsed);
        return { ok: true };
      } catch {
        return { ok: false, error: "JSON invalide." };
      }
    },
  },
});
