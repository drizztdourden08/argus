/* @layer renderer-app @kind types */
import type { CacheReport } from '../../../../data/cache';
import type { CacheTableRow } from './model';

interface CacheReportState {
  report: CacheReport | null;
  error: string;
  reload: () => void;
}

interface PruneFlow {
  asking: boolean;
  busy: boolean;
  ask: () => void;
  cancel: () => void;
  confirm: () => void;
}

interface CacheSummaryProps {
  report: CacheReport;
}

interface CacheNoticesProps {
  report: CacheReport;
  error: string;
}

interface CacheTableProps {
  report: CacheReport;
}

interface CacheCellProps {
  row: CacheTableRow;
  path: string;
}

interface UsersCellProps {
  row: CacheTableRow;
}

interface PruneBarProps {
  report: CacheReport;
  flow: PruneFlow;
}

interface PruneConfirmProps {
  report: CacheReport;
  flow: PruneFlow;
}

export type { CacheCellProps, CacheNoticesProps, CacheReportState, CacheSummaryProps, CacheTableProps, PruneBarProps, PruneConfirmProps, PruneFlow, UsersCellProps };
