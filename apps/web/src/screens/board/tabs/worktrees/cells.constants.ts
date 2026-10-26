/* @layer renderer-app @kind constants */
import type { ComponentType } from 'react';
import { ActionsCell } from './sub-components/ActionsCell';
import { AheadCell } from './sub-components/AheadCell';
import { BranchCell } from './sub-components/BranchCell';
import { DirtyCell } from './sub-components/DirtyCell';
import { PathCell } from './sub-components/PathCell';
import { RequestCell } from './sub-components/RequestCell';
import { StatusCell } from './sub-components/StatusCell';
import { SubjectCell } from './sub-components/SubjectCell';
import { TouchedCell } from './sub-components/TouchedCell';
import type { CellProps } from './worktrees.type';

const CELLS: Readonly<Record<string, ComponentType<CellProps>>> = {
  path: PathCell,
  status: StatusCell,
  branch: BranchCell,
  touched: TouchedCell,
  ahead: AheadCell,
  dirty: DirtyCell,
  subject: SubjectCell,
  request: RequestCell,
  actions: ActionsCell,
};

export { CELLS };
