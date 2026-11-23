/* @layer renderer-app @kind types */
import type { IconName, SignalTone, SortDirection } from '@drizztdourden08/tessera';
import type { WithId } from '../../../../data/react';
import type { CheckRun } from '../../../../data/remote';
import type { PullRequest } from '../../../../data/types';
import type { ItemRef } from '../../../../data/kanban';
import type { GroupThreads } from '../../item-groups';

interface LogTarget {
  runId: string;
  index: number;
  name: string;
  command: string;
}

type PullRequestDoc = WithId<PullRequest>;

interface ChecksProgress {
  passed: number;
  total: number;
  failed: boolean;
}

interface PrSignal {
  id: string;
  icon: IconName;
  label: string;
  tone: SignalTone;
}

interface ChangesTabProps {
  boardKey: string;
}

type ChangesGroup = 'item' | 'status' | 'none';

type ChangesLayout = 'normal' | 'compact';

interface ChangesLayoutProps {
  value: ChangesLayout;
  onChange: (layout: ChangesLayout) => void;
}

type ChangesOrder = 'created' | 'merged' | 'title';

interface ChangesFilters {
  query: string;
  status: string;
  item: string;
  author: string;
  base: string;
}

interface ChangesView {
  group: ChangesGroup;
  setGroup: (group: ChangesGroup) => void;
  order: ChangesOrder;
  direction: SortDirection;
  setSort: (order: ChangesOrder, direction: SortDirection) => void;
  filters: ChangesFilters;
  patchFilters: (values: Partial<ChangesFilters>) => void;
}

interface ChangesToolsProps {
  boardKey: string;
  view: ChangesView;
  prs: readonly PullRequestDoc[];
  items: readonly ItemRef[];
  layout: ChangesLayoutProps;
}

interface PrGroupsProps {
  boardKey: string;
  prs: readonly PullRequestDoc[];
  group: ChangesGroup;
  compact: boolean;
}

interface PrListProps {
  boardKey: string;
  rows: readonly PullRequestDoc[];
  threads: GroupThreads;
  compact: boolean;
}

interface PrCardProps {
  boardKey: string;
  pr: PullRequestDoc;
  threads: GroupThreads;
  compact: boolean;
}

interface PrFootProps {
  pr: PullRequestDoc;
  runs: readonly CheckRun[] | null;
  compact: boolean;
}

interface PrSignalsProps {
  pr: PullRequestDoc;
  runs: readonly CheckRun[] | null;
}

interface StepListProps {
  run: CheckRun;
  onLog: (target: LogTarget) => void;
}

interface StepLogBodyProps {
  boardKey: string;
  target: LogTarget;
}

export type {
  ChangesFilters, ChangesGroup, ChangesLayout, ChangesOrder, ChangesTabProps, ChangesToolsProps, ChecksProgress, LogTarget, PrCardProps, PrFootProps, PrGroupsProps, PrListProps, PrSignal, PrSignalsProps, PullRequestDoc, StepListProps, StepLogBodyProps,
};
