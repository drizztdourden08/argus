/* @layer renderer-app @kind types */
import type { KanbanColumnConfig, KanbanConfig, KanbanRuleConfig } from '../../../../data/types';
import type { ParamField } from './model';

interface ConfigEditor {
  config: KanbanConfig;
  custom: boolean;
  dirty: boolean;
  busy: boolean;
  problem: string | null;
  edit: (change: (config: KanbanConfig) => KanbanConfig) => void;
  save: () => Promise<void>;
  useDefault: () => Promise<void>;
  discard: () => void;
}

interface ConfigSectionProps {
  boardKey: string;
}

interface ConfigColumnRowProps {
  column: KanbanColumnConfig;
  index: number;
  count: number;
  rules: readonly KanbanRuleConfig[];
  onChange: (column: KanbanColumnConfig) => void;
  onMove: (step: -1 | 1) => void;
  onRemove: () => void;
}

interface ConfigColumnFieldsProps {
  column: KanbanColumnConfig;
  rules: readonly KanbanRuleConfig[];
  onChange: (column: KanbanColumnConfig) => void;
}

interface ConfigColumnLimitProps {
  column: KanbanColumnConfig;
  onChange: (column: KanbanColumnConfig) => void;
}

interface ConfigRuleRowProps {
  rule: KanbanRuleConfig;
  onChange: (rule: KanbanRuleConfig) => void;
  onRemove: () => void;
}

interface ConfigRuleFieldProps {
  rule: KanbanRuleConfig;
  onChange: (rule: KanbanRuleConfig) => void;
}

interface ConfigHoursInputProps {
  label: string;
  ms: number;
  onChange: (ms: number) => void;
}

interface ConfigListInputProps {
  label: string;
  value: readonly string[];
  hint?: string;
  onCommit: (value: string[]) => void;
}

interface ConfigRuleParamProps extends ConfigRuleFieldProps {
  field: ParamField;
}

export type { ConfigRuleParamProps, ConfigHoursInputProps, ConfigListInputProps, ConfigColumnFieldsProps, ConfigColumnLimitProps, ConfigColumnRowProps, ConfigEditor, ConfigRuleFieldProps, ConfigRuleRowProps, ConfigSectionProps };
