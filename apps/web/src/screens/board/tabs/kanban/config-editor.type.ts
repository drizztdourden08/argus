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

interface ConfigColumnProps {
  column: KanbanColumnConfig;
  index: number;
  count: number;
  rules: readonly KanbanRuleConfig[];
  onChange: (column: KanbanColumnConfig) => void;
  onMove: (step: -1 | 1) => void;
  onRemove: () => void;
  onRuleChange: (index: number, rule: KanbanRuleConfig) => void;
  onRuleRemove: (index: number) => void;
}

interface ConfigColumnFieldsProps {
  column: KanbanColumnConfig;
  onChange: (column: KanbanColumnConfig) => void;
}

type ConfigColumnRulesProps = Pick<ConfigColumnProps, 'column' | 'rules' | 'onChange' | 'onRuleChange' | 'onRuleRemove'>;

interface ConfigColumnLimitProps {
  column: KanbanColumnConfig;
  onChange: (column: KanbanColumnConfig) => void;
}

interface ConfigRuleCardProps {
  rule: KanbanRuleConfig;
  detachLabel?: string;
  onChange: (rule: KanbanRuleConfig) => void;
  onRemove: () => void;
  onDetach?: () => void;
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

export type { ConfigRuleParamProps, ConfigHoursInputProps, ConfigListInputProps, ConfigColumnFieldsProps, ConfigColumnLimitProps, ConfigColumnProps, ConfigColumnRulesProps, ConfigEditor, ConfigRuleFieldProps, ConfigRuleCardProps, ConfigSectionProps };
