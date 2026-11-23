/* @layer renderer-app @kind types */
type KanbanLimitMode = 'warn' | 'refuse' | 'alert';

interface KanbanColumnLimit {
  max: number;
  mode: KanbanLimitMode;
}

interface KanbanColumnConfig {
  id: string;
  name: string;
  accent?: string | null;
  match: { lanes: string[]; stages?: string[] | null };
  limit?: KanbanColumnLimit | null;
  rules?: string[];
}

interface KanbanRuleConfig {
  id: string;
  kind: string;
  params?: { ms?: number } & Record<string, unknown>;
}

interface KanbanConfig {
  columns: KanbanColumnConfig[];
  rules?: KanbanRuleConfig[];
}

export type { KanbanColumnConfig, KanbanColumnLimit, KanbanConfig };
