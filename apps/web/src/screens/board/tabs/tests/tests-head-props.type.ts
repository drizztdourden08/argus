/* @layer renderer-app @kind types */
import type { IconName } from '@drizztdourden08/tessera';
import type { RecordFrameProps } from '../../../../shell';
import type { ReactNode } from 'react';
import type { RunEntry } from '../../../../data/runs';
import type { RunView, TestDoc } from './model';
import type { CaseHead } from './tests.type';

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

interface RunFrameProps extends Pick<RecordFrameProps, 'className' | 'edgeTone'> {
  view: RunView;
  muted?: boolean;
  open: boolean;
  onToggle: (open: boolean) => void;
  menu?: ReactNode;
  signals: ReactNode;
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

export type {
  BuildChipProps, CaseOutcomeProps, RunActionsProps, RunChipsProps, RunFrameProps, SectionHeadProps,
};
