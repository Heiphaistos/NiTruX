export type StyleId =
  | "glass-glow" | "flat-modern" | "neon-terminal" | "neumorphism"
  | "brutalism" | "paper" | "aero-glass" | "cyber-grid"
  | "line-art" | "gradient-mesh" | "amber-crt" | "mono-contrast";

export interface StyleDefinition {
  id: StyleId;
  name: string;
  description: string;
}
