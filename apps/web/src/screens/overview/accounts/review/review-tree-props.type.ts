/* @layer renderer-app @kind types */
import type { IconName } from '@drizztdourden08/tessera';
import type { ReactNode } from 'react';
import type { ReviewAccount, ReviewRow } from '../../../../data/types';
import type { ChooseFn, Decisions, EditStager, Pair, RowActFn, Side } from './review.type';

interface TreeEnv {
  decisions: Decisions;
  collapsed: ReadonlySet<string>;
  pending: ReadonlySet<string>;
  folders: ReadonlyMap<string, readonly string[]>;
  scopes: ReadonlyMap<string, readonly ReviewRow[]>;
  pair: Pair;
  now: number;
  onHover: (rowId: string | null) => void;
  onToggle: (rowId: string) => void;
  onAct: RowActFn;
  onStage: EditStager;
  onChoose: ChooseFn;
}

interface ReviewTreeProps {
  accounts: readonly ReviewAccount[];
  rows: readonly ReviewRow[];
  hovered: string | null;
  env: TreeEnv;
}

interface TreePanelProps {
  account: ReviewAccount;
  rows: readonly ReviewRow[];
  hovered: string | null;
  env: TreeEnv;
}

interface TreeGutterProps {
  pair: Pair;
  rows: readonly ReviewRow[];
  hovered: string | null;
  env: TreeEnv;
}

interface TreeRowProps {
  row: ReviewRow;
  account: string;
  hot: boolean;
  env: TreeEnv;
}

interface TreeLeadProps {
  row: ReviewRow;
  ghost: boolean;
  kids: boolean;
  collapsed: boolean;
  onToggle: () => void;
}

interface RowTitleProps {
  row: ReviewRow;
  title: string;
  ghost: boolean;
  muted: boolean;
  count: number | null;
  onToggle?: () => void;
}

interface TreeStepsProps {
  depth: number;
}

interface RowActionsProps {
  row: ReviewRow;
  account: string;
  env: TreeEnv;
}

interface ScopeMenuArgs {
  row: ReviewRow;
  account: string;
  env: TreeEnv;
}

interface ScopeMenuProps extends ScopeMenuArgs {
  label: string;
}

interface GutterCellProps {
  row: ReviewRow;
  pair: Pair;
  hot: boolean;
  env: TreeEnv;
}

interface SideButtonProps {
  side: Side;
  rows: readonly ReviewRow[];
  pair: Pair;
  env: TreeEnv;
}

interface ChoiceMenuProps {
  row: ReviewRow;
  pair: Pair;
  env: TreeEnv;
}

interface ActionButtonProps {
  label: string;
  icon: IconName;
  danger?: boolean;
  onClick: () => void;
}

interface RowTipProps {
  label: string;
  children: ReactNode;
}

interface ExpandBarProps {
  onExpandAll: () => void;
  onCollapseAll: () => void;
}

export type {
  ActionButtonProps, ChoiceMenuProps, ExpandBarProps, GutterCellProps, ReviewTreeProps, RowActionsProps, RowTipProps, RowTitleProps, ScopeMenuArgs, ScopeMenuProps, SideButtonProps, TreeEnv, TreeGutterProps, TreeLeadProps, TreePanelProps, TreeRowProps, TreeStepsProps,
};
