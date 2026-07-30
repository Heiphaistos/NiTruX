export type LayoutId =
  | "sidebar-classic" | "widgets-grid" | "command-first" | "compact-sidebar"
  | "top-nav" | "master-detail" | "bento" | "floating-dock";

export interface LayoutDefinition {
  id: LayoutId;
  name: string;
  description: string;
}
