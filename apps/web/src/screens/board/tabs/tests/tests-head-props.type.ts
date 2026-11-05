/* @layer renderer-app @kind types */
import type { IconButtonProps, IconName } from '@drizztdourden08/tessera';
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

type TipButtonProps = Omit<IconButtonProps, 'children'> & {
  icon: IconName;
  tip: ReactNode;
};

interface RunTitleProps {
  view: RunView;
  muted?: boolean;
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
  BuildChipProps, CaseOutcomeProps, LayoutControlProps, RunActionsProps, RunChipsProps, RunTitleProps, SectionHeadProps, TestsSummaryProps, TipButtonProps,
};
