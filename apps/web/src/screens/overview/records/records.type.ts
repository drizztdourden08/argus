/* @layer renderer-app @kind types */
import type { ReactNode } from 'react';
import type { ActionHandlers, RowChip } from '../../../shell/records';

interface RowTarget {
  kind: string;
  code: string;
  project: string;
  open: string | null;
  tab?: string;
}

interface OverviewRowProps {
  target: RowTarget;
  assignee: string;
  title: string;
  chips?: readonly RowChip[];
  trailing?: ReactNode;
  handlers?: ActionHandlers;
}

interface OverviewListProps {
  label: string;
  children: ReactNode;
}

export type { OverviewListProps, OverviewRowProps, RowTarget };
