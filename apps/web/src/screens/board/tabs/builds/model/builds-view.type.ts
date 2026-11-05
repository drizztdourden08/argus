/* @layer renderer-app @kind types */
import type { SortDirection } from '@drizztdourden08/tessera';
import type { LauncherView } from './model.type';

type BuildGroup = 'item' | 'kind' | 'none';

type BuildOrder = 'updated' | 'name';

type BuildFilterKey = 'item' | 'kind' | 'stage';

interface BuildFilters {
  query: string;
  item: string[];
  kind: string[];
  stage: string[];
}

interface BuildViewSpec {
  filters: BuildFilters;
  order: BuildOrder;
  direction: SortDirection;
}

interface KindBuilds {
  key: string;
  label: string;
  views: LauncherView[];
}

export type { BuildFilterKey, BuildFilters, BuildGroup, BuildOrder, BuildViewSpec, KindBuilds };
