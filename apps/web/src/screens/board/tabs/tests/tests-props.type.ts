/* @layer renderer-app @kind types */
import type { TestOutcome } from '../../../../data/posts';
import type { RunEntry } from '../../../../data/runs';
import type { RecordGroups } from '../../../../shell';
import type { Counts, GroupCard, ItemDoc, PlanView, RunView, TestDoc, TestNotice, TestsView } from './model';
import type { NoticeOps, Outcome, TestsState } from './tests.type';

interface TestsTabProps {
  boardKey: string;
}

interface Option {
  value: string;
  label: string;
}

interface TestsToolbarProps {
  boardKey: string;
  state: TestsState;
  threads: Option[];
  subjects: string[];
  runs: Option[];
  plans: Option[];
}

interface TestNoticesProps {
  label: string;
  notices: TestNotice[];
  ops: NoticeOps;
}

interface RunCardProps {
  view: RunView;
  runs: RunEntry[];
  open: boolean;
  onToggle: (open: boolean) => void;
}

interface RunProgressProps {
  counts: Counts;
  fill?: boolean;
}

interface HandButtonProps {
  run: ItemDoc;
  handed: boolean;
}

interface TestSectionsProps {
  tests: TestDoc[];
  emptyText: string;
}

interface TestPartProps {
  test: TestDoc;
}

interface DoneSummaryProps {
  test: TestDoc;
}

interface OutcomeControlsProps {
  test: TestDoc;
  controls: Outcome;
}

interface OutcomeButtonsProps {
  testId: string;
  active: TestOutcome | null;
  busy: boolean;
  onChoose: (outcome: TestOutcome) => void;
}

interface ResultComposerProps {
  testId: string;
  outcome: TestOutcome;
  controls: Outcome;
}

interface RerunButtonProps {
  test: TestDoc;
  boardKey: string;
}

interface ClosedRunCardProps {
  view: RunView;
  open: boolean;
  onToggle: (open: boolean) => void;
}

interface DoneListProps {
  tests: TestDoc[];
}

interface ClosedSectionProps {
  views: RunView[];
  count: number;
  shown: boolean;
  onShownChange: (shown: boolean) => void;
}

interface RunListProps {
  views: RunView[];
  runs: RunEntry[];
  filtered: boolean;
}

interface GroupListProps {
  cards: GroupCard[];
  filtered: boolean;
}

interface TestCardProps {
  card: GroupCard;
}

interface PlanSectionsProps {
  view: TestsView;
}

interface PlanCardsProps {
  plans: PlanView[];
}

interface PlanCardProps {
  plan: PlanView;
}

interface PlanFootProps {
  plan: PlanView;
  code: string;
}

interface PlanGroupsProps {
  groups: RecordGroups<TestDoc>;
  plans: PlanView[];
}

interface PlanGroupProps {
  plan: PlanView;
  tests: TestDoc[];
}

interface CaseListProps {
  tests: TestDoc[];
}

interface PlanHeaderProps {
  plan: PlanView;
}

export type {
  TestsTabProps, RunCardProps, RunListProps, RunProgressProps, ClosedRunCardProps, CaseListProps, ClosedSectionProps, DoneListProps, DoneSummaryProps, GroupListProps, HandButtonProps, OutcomeButtonsProps, OutcomeControlsProps, PlanCardProps, PlanCardsProps, PlanFootProps, PlanGroupProps, PlanGroupsProps, PlanHeaderProps, PlanSectionsProps, RerunButtonProps, ResultComposerProps, TestCardProps, TestNoticesProps, TestPartProps, TestSectionsProps, TestsToolbarProps,
};
