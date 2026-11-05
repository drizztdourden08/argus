/* @layer renderer-app @kind types */
import type { IconName } from '@drizztdourden08/tessera';
import type { ReactNode } from 'react';
import type { RunEntry } from '../../../../data/runs';
import type { RunView, TestDoc, TestsView } from './model';
import type { CaseHead, CaseLayout } from './tests.type';

interface CaseOutcomeProps {
  test: TestDoc;
  view: CaseHead;
}

interface SectionHeadProps {
  icon: IconName;
  title: string;
  count: number;
  action?: ReactNode;
}

interface RunFrameProps {
  view: RunView;
  muted?: boolean;
  open: boolean;
  onToggle: (open: boolean) => void;
  content: ReactNode;
  children: ReactNode;
}

interface RunChipsProps {
  view: RunView;
}

interface BuildChipProps {
  build: string;
  gone: string;
}

interface RunActionsProps {
  view: RunView;
  runs: RunEntry[];
}

interface LayoutControlProps {
  layout: CaseLayout;
  onChange: (layout: CaseLayout) => void;
}

interface TestsSummaryProps {
  view: TestsView;
}

export type {
  BuildChipProps, CaseOutcomeProps, LayoutControlProps, RunActionsProps, RunChipsProps, RunFrameProps, SectionHeadProps, TestsSummaryProps,
};
