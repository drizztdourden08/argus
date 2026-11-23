/* @layer renderer-app @kind types */
type ParamType = 'list' | 'hours' | 'text';

interface ParamField {
  key: string;
  label: string;
  type: ParamType;
}

interface RuleKind {
  kind: string;
  sweep: boolean;
  params: readonly ParamField[];
}

export type { ParamField, RuleKind };
