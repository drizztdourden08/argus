/* @layer renderer-app @kind types */
import type { ReactNode } from 'react';

interface RuleFact {
  label: string;
  value: string;
}

interface RuleCardToggle {
  label: string;
  checked: boolean;
  disabled?: boolean;
  onChange: (on: boolean) => void;
}

interface RuleCardProps {
  title: string;
  kind: string;
  badge?: string;
  facts: readonly RuleFact[];
  reaction: string;
  toggle?: RuleCardToggle;
  actions?: ReactNode;
  editing?: ReactNode;
  editLabel: string;
  removeLabel: string;
  busy?: boolean;
  onEdit: () => void;
  onRemove: () => void;
}

export type { RuleCardProps, RuleCardToggle, RuleFact };
