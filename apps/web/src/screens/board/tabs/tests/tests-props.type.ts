/* @layer renderer-app @kind types */
import type { ReactNode } from 'react';
import type { TestOutcome } from '../../../../data/posts';
import type { WithId } from '../../../../data/react';
import type { RunEntry } from '../../../../data/runs';
import type { BundleState, TestItem } from '../../../../data/types';
import type { BundleGroup, ClosedGroup, Counts, TestDoc } from './model';
import type { BundleView, Folds, Outcome } from './tests.type';

interface TestsTabProps {
  boardKey: string;
}

interface OpenTest {
  onOpenTest: (testId: string) => void;
}

interface BundleCardProps extends OpenTest {
  view: BundleView;
  boardKey: string;
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
  boardKey: string;
  runs: RunEntry[];
  state: BundleState;
}

interface HandButtonProps {
  boardKey: string;
  tag: string;
  handed: boolean;
}

interface TestSectionsProps extends OpenTest {
  tests: TestDoc[];
  emptyText: string;
  boardKey: string;
}

interface TestRowProps extends OpenTest {
  test: TestDoc;
  boardKey: string;
}

interface TestChipsProps {
  test: TestDoc;
}

interface DoneSummaryProps {
  test: TestDoc;
}

interface OutcomeControlsProps {
  test: TestDoc;
  boardKey: string;
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

interface TestDrawerProps {
  test: TestDoc | null;
  item: WithId<TestItem> | null;
  boardKey: string;
  handed: boolean;
  onClose: () => void;
}

interface TestFactsProps {
  test: TestDoc;
  handed: boolean;
}

interface ResultBlockProps {
  test: TestDoc;
}

interface TestPartProps {
  test: TestDoc;
}

interface TestBodyProps {
  test: TestDoc;
  item: WithId<TestItem> | null;
  boardKey: string;
  handed: boolean;
}

interface AcListProps {
  test: TestDoc;
  item: WithId<TestItem> | null;
}

interface EvidenceProps {
  test: TestDoc;
  boardKey: string;
}

interface ProofUploadFormProps {
  testId: string;
  boardKey: string;
}

interface ResultHistoryProps {
  runs: TestDoc['runs'];
}

interface LaunchCommandProps {
  view: BundleView;
  boardKey: string;
  runs: RunEntry[];
  draft: boolean;
}

interface ShotPreviewProps {
  file: File;
  testId: string;
  onRemove: () => void;
}

interface DoneListProps extends OpenTest {
  tests: TestDoc[];
  boardKey: string;
}

interface DrawerSectionProps {
  title: string;
  count?: number;
  children: ReactNode;
}

interface ClosedSectionProps {
  groups: ClosedGroup[];
  folds: Folds;
}

interface BundleListProps extends OpenTest {
  bundles: BundleView[];
  boardKey: string;
  runs: RunEntry[];
  folds: Folds;
}

interface TextListProps {
  lines: string[];
  ordered?: boolean;
  label: string;
}

export type {
  TestsTabProps, AcListProps, BundleCardProps, BundleHeadProps, BundleListProps, BundleProgressProps, ClosedBundleCardProps, ClosedSectionProps, DoneListProps, DoneSummaryProps, DrawerSectionProps, EvidenceProps, HandButtonProps, LaunchBoxProps, LaunchCommandProps, OutcomeButtonsProps, OutcomeControlsProps, ProofUploadFormProps, RerunButtonProps, ResultBlockProps, ResultComposerProps, ResultHistoryProps, ShotPreviewProps, TestBodyProps, TestChipsProps, TestDrawerProps, TestFactsProps, TestPartProps, TestRowProps, TestSectionsProps, TextListProps,
};
