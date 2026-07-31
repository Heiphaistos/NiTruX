export interface ThemeColors {
  bgBase: string;
  bgElevated: string;
  bgOverlay: string;
  border: string;
  textPrimary: string;
  textSecondary: string;
  accentPrimary: string;
  accentSecondary: string;
  accentSuccess: string;
  accentWarning: string;
  accentDanger: string;
}

export interface Theme {
  id: string;
  name: string;
  mode: "dark" | "light";
  colors: ThemeColors;
}
