import type { Theme } from "@/types/theme";

export const builtinThemes: Theme[] = [
  {
    id: "catppuccin-mocha", name: "Catppuccin Mocha", mode: "dark",
    colors: {
      bgBase: "#1e1e2e", bgElevated: "#313244", bgOverlay: "#181825", border: "#45475a",
      textPrimary: "#cdd6f4", textSecondary: "#a6adc8",
      accentPrimary: "#89b4fa", accentSecondary: "#f5c2e7",
      accentSuccess: "#94e2d5", accentWarning: "#fab387", accentDanger: "#f38ba8",
    },
  },
  {
    id: "nord", name: "Nord", mode: "dark",
    colors: {
      bgBase: "#2e3440", bgElevated: "#3b4252", bgOverlay: "#242933", border: "#4c566a",
      textPrimary: "#eceff4", textSecondary: "#d8dee9",
      accentPrimary: "#88c0d0", accentSecondary: "#5e81ac",
      accentSuccess: "#a3be8c", accentWarning: "#ebcb8b", accentDanger: "#bf616a",
    },
  },
  {
    id: "adwaita", name: "Adwaita", mode: "light",
    colors: {
      bgBase: "#fafafa", bgElevated: "#ffffff", bgOverlay: "#f0f0f0", border: "#d8d8d8",
      textPrimary: "#241f31", textSecondary: "#5e5c64",
      accentPrimary: "#3584e4", accentSecondary: "#9141ac",
      accentSuccess: "#2ec27e", accentWarning: "#e5a50a", accentDanger: "#e01b24",
    },
  },
  {
    id: "gruvbox", name: "Gruvbox", mode: "dark",
    colors: {
      bgBase: "#282828", bgElevated: "#3c3836", bgOverlay: "#1d2021", border: "#504945",
      textPrimary: "#ebdbb2", textSecondary: "#bdae93",
      accentPrimary: "#fe8019", accentSecondary: "#d3869b",
      accentSuccess: "#b8bb26", accentWarning: "#fabd2f", accentDanger: "#fb4934",
    },
  },
  {
    id: "dracula", name: "Dracula", mode: "dark",
    colors: {
      bgBase: "#282a36", bgElevated: "#44475a", bgOverlay: "#21222c", border: "#6272a4",
      textPrimary: "#f8f8f2", textSecondary: "#bfbfd4",
      accentPrimary: "#bd93f9", accentSecondary: "#ff79c6",
      accentSuccess: "#50fa7b", accentWarning: "#f1fa8c", accentDanger: "#ff5555",
    },
  },
  {
    id: "everforest", name: "Everforest", mode: "dark",
    colors: {
      bgBase: "#2d353b", bgElevated: "#3d484d", bgOverlay: "#232a2e", border: "#4f5b58",
      textPrimary: "#d3c6aa", textSecondary: "#a6b0a0",
      accentPrimary: "#a7c080", accentSecondary: "#dbbc7f",
      accentSuccess: "#83c092", accentWarning: "#e69875", accentDanger: "#e67e80",
    },
  },
  {
    id: "tokyo-night", name: "Tokyo Night", mode: "dark",
    colors: {
      bgBase: "#1a1b26", bgElevated: "#24283b", bgOverlay: "#16161e", border: "#3b4261",
      textPrimary: "#c0caf5", textSecondary: "#9aa5ce",
      accentPrimary: "#7aa2f7", accentSecondary: "#bb9af7",
      accentSuccess: "#9ece6a", accentWarning: "#e0af68", accentDanger: "#f7768e",
    },
  },
  {
    id: "solarized", name: "Solarized", mode: "light",
    colors: {
      bgBase: "#fdf6e3", bgElevated: "#eee8d5", bgOverlay: "#e4ddc4", border: "#93a1a1",
      textPrimary: "#073642", textSecondary: "#586e75",
      accentPrimary: "#268bd2", accentSecondary: "#6c71c4",
      accentSuccess: "#2aa198", accentWarning: "#b58900", accentDanger: "#dc322f",
    },
  },
  {
    id: "rose-pine", name: "Rosé Pine", mode: "dark",
    colors: {
      bgBase: "#191724", bgElevated: "#1f1d2e", bgOverlay: "#26233a", border: "#403d52",
      textPrimary: "#e0def4", textSecondary: "#908caa",
      accentPrimary: "#c4a7e7", accentSecondary: "#ebbcba",
      accentSuccess: "#9ccfd8", accentWarning: "#f6c177", accentDanger: "#eb6f92",
    },
  },
  {
    id: "one-dark", name: "One Dark", mode: "dark",
    colors: {
      bgBase: "#282c34", bgElevated: "#2c313a", bgOverlay: "#21252b", border: "#3e4451",
      textPrimary: "#abb2bf", textSecondary: "#828997",
      accentPrimary: "#61afef", accentSecondary: "#c678dd",
      accentSuccess: "#98c379", accentWarning: "#e5c07b", accentDanger: "#e06c75",
    },
  },
  {
    id: "kanagawa", name: "Kanagawa", mode: "dark",
    colors: {
      bgBase: "#1f1f28", bgElevated: "#2a2a37", bgOverlay: "#16161d", border: "#54546d",
      textPrimary: "#dcd7ba", textSecondary: "#a6a69c",
      accentPrimary: "#7e9cd8", accentSecondary: "#957fb8",
      accentSuccess: "#98bb6c", accentWarning: "#e6c384", accentDanger: "#ff5d62",
    },
  },
  {
    id: "ayu", name: "Ayu", mode: "dark",
    colors: {
      bgBase: "#0a0e14", bgElevated: "#131721", bgOverlay: "#060a10", border: "#232834",
      textPrimary: "#b3b1ad", textSecondary: "#828282",
      accentPrimary: "#39bae6", accentSecondary: "#ffb454",
      accentSuccess: "#c2d94c", accentWarning: "#ffb454", accentDanger: "#f07178",
    },
  },
];
