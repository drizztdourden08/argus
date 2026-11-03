/* @layer renderer-app @kind types */
import type { DepDoc, DepEnd } from '../../../data/deps';
import type { WithId } from '../../../data/react';
import type { DepFilter } from './model';

interface DepsFilterProps {
  value: DepFilter;
  onChange: (value: DepFilter) => void;
}

interface DepsListProps {
  docs: readonly WithId<DepDoc>[];
  all: readonly WithId<DepDoc>[];
  now: number;
}

interface DepRowProps {
  doc: WithId<DepDoc>;
  all: readonly WithId<DepDoc>[];
  now: number;
}

interface DepWaitProps {
  doc: WithId<DepDoc>;
  now: number;
}

interface DepEndCellProps {
  end: DepEnd;
}

export type { DepEndCellProps, DepRowProps, DepsFilterProps, DepsListProps, DepWaitProps };
