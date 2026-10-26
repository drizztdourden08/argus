/* @layer renderer-app @kind types */
import type { TestOutcome } from '../../../../data/posts';
import type { RunEntry } from '../../../../data/runs';
import type { BundleState } from '../../../../data/types';
import type { BundleGroup, BundleView, ClosedGroup, Counts, GroupCard, TestDoc, TestNotice, TestsView } from './model';
import type { NoticeOps, Outcome, RowMenu, TestsState } from './tests.type';

interface TestsTabProps {
  boardKey: string;
}

interface Option {
  value: string;
  label: string;
}

interface MultiFilterProps {
  label: string;
  options: Option[];
  values: string[];
  onPick: (values: string[]) => void;
}

interface ChoiceFilterProps {
  label: string;
  any: string;
  options: Option[];
  value: string;
  onPick: (value: string) => void;
}

interface ClearFiltersProps {
  state: TestsState;
}

interface TestsToolbarProps {
  state: TestsState;
  view: TestsView;
  threads: Option[];
  subjects: string[];
  builds: string[];
  handed: string | null;
}

interface FilterRowProps {
  state: TestsState;
  threads: Option[];
  subjects: string[];
  builds: string[];
  handed: string | null;
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

interface BundleProgressProps {
  counts: Counts;
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

interface TestPartProps {
  test: TestDoc;
}

interface TestMenuProps extends TestPartProps {
  menu: RowMenu;
}

interface TestChipsProps {
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

export type {
  TestsTabProps, BundleCardProps, ChoiceFilterProps, MultiFilterProps, BundleHeadProps, BundleListProps, BundleProgressProps, ClearFiltersProps, ClosedBundleCardProps, ClosedSectionProps, DoneListProps, DoneSummaryProps, FilterRowProps, GroupListProps, HandButtonProps, LaunchBoxProps, LaunchCommandProps, LaunchNoteProps, OutcomeButtonsProps, OutcomeControlsProps, RerunButtonProps, ResultComposerProps, TestCardProps, TestChipsProps, TestMenuProps, TestNoticesProps, TestPartProps, TestSectionsProps, TestsToolbarProps, ViewRowProps,
};
