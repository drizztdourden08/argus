/* @layer renderer-app @kind types */
import type { SortDirection } from '@drizztdourden08/tessera';
import type { ReleaseChannel } from '../../../../data/remote';

interface ReleasesTabProps {
  boardKey: string;
}

type ReleasesLayout = 'normal' | 'compact';

type ReleasesOrder = 'newest' | 'channel';

interface ReleasesFilters {
  query: string;
  channel: string;
}

interface ReleasesView {
  order: ReleasesOrder;
  direction: SortDirection;
  setSort: (order: ReleasesOrder, direction: SortDirection) => void;
  filters: ReleasesFilters;
  patchFilters: (values: Partial<ReleasesFilters>) => void;
}

interface ReleasesToolsProps {
  boardKey: string;
  layout: { value: ReleasesLayout; onChange: (layout: ReleasesLayout) => void };
  view: ReleasesView;
  channels: readonly ReleaseChannel[];
}

export type { ReleasesFilters, ReleasesLayout, ReleasesOrder, ReleasesTabProps, ReleasesToolsProps };
