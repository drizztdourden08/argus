/* @layer renderer-app @kind types */
import type { IconName, MenuGroup } from '@drizztdourden08/tessera';
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

interface RunFrameProps {
  view: RunView;
  open: boolean;
  onToggle: (open: boolean) => void;
  menu: MenuGroup[];
  signals: ReactNode;
  className?: string;
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
