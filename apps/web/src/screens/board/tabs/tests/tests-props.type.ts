/* @layer renderer-app @kind types */
import type { ReactNode } from 'react';
import type { TestOutcome } from '../../../../data/posts';
import type { RunEntry } from '../../../../data/runs';
import type { BundleState } from '../../../../data/types';
import type { RecordGroups } from '../../../../shell';
import type { BundleGroup, BundleView, ClosedGroup, Counts, GroupCard, PlanView, TestDoc, TestNotice, TestsView } from './model';
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
  view: TestsView;
  threads: Option[];
  subjects: string[];
  plans: Option[];
  builds: string[];
  handed: string | null;
}

interface ViewMenuProps {
  state: TestsState;
}

interface ViewRowProps {
  state: TestsState;
  view: TestsView;
}

interface TestNoticesProps {
  label: string;
  notices: TestNotice[];
  ops: NoticeOps;
}

interface BundleCardProps {
  view: BundleView;
  runs: RunEntry[];
  open: boolean;
  onToggle: (open: boolean) => void;
}

interface BundleHeadProps {
  group: BundleGroup;
  state: BundleState;
}

interface BundleNameProps {
  tag: string;
  title?: string | null;
  muted?: boolean;
}

interface BundleTagsProps {
  group: BundleGroup;
  state: BundleState;
}

interface BundleProgressProps {
  counts: Counts;
  fill?: boolean;
}

interface LaunchBoxProps {
  view: BundleView;
  runs: RunEntry[];
  state: BundleState;
}

interface LaunchNoteProps {
  tag: string;
  note: string;
}

interface HandButtonProps {
  tag: string;
  handed: boolean;
}

interface TestSectionsProps {
  tests: TestDoc[];
  emptyText: string;
}

interface SectionBandProps {
  children: ReactNode;
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

interface ClosedBundleCardProps {
  group: ClosedGroup;
  open: boolean;
  onToggle: (open: boolean) => void;
}

interface LaunchCommandProps {
  view: BundleView;
  runs: RunEntry[];
  draft: boolean;
}

interface DoneListProps {
  tests: TestDoc[];
}

interface ClosedSectionProps {
  groups: ClosedGroup[];
  count: number;
  shown: boolean;
  onShownChange: (shown: boolean) => void;
}

interface BundleListProps {
  bundles: BundleView[];
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
  TestsTabProps, BundleCardProps, BundleHeadProps, BundleListProps, BundleNameProps, BundleProgressProps, BundleTagsProps, ClosedBundleCardProps, CaseListProps, ClosedSectionProps, DoneListProps, DoneSummaryProps, GroupListProps, HandButtonProps, LaunchBoxProps, LaunchCommandProps, LaunchNoteProps, OutcomeButtonsProps, OutcomeControlsProps, PlanCardProps, PlanCardsProps, PlanFootProps, PlanGroupProps, PlanGroupsProps, PlanHeaderProps, PlanSectionsProps, RerunButtonProps, ResultComposerProps, SectionBandProps, TestCardProps, TestNoticesProps, TestPartProps, TestSectionsProps, TestsToolbarProps, ViewMenuProps, ViewRowProps,
};
