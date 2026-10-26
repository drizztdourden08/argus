/* @layer renderer-app @kind component */
import { Inline, SegmentedControl } from '@drizztdourden08/tessera';
import type { SegmentOption } from '@drizztdourden08/tessera';
import { ChipFilter } from '../../sub-components/ChipFilter';
import type { ActivityFilterBarProps } from '../activity.type';
import { LEVELS } from '../model';
import type { LevelFilter } from '../model';

const ActivityFilterBar = (props: ActivityFilterBarProps) => {
  const { filters, levels, kinds, total, onChange } = props;
  const levelOptions: SegmentOption<LevelFilter>[] = [
    { value: 'all', label: `All levels ${levels.all}` },
    ...LEVELS.map((level) => ({ value: level, label: `${level} ${levels[level]}` })),
  ];
  const kindOptions = kinds.map(([kind, count]) => ({ value: kind, label: `${kind} ${count}` }));
  return (
    <Inline gap="md" align="center" wrap>
      <SegmentedControl<LevelFilter> aria-label="Level" value={filters.level} options={levelOptions} onChange={(level) => onChange({ ...filters, level })} />
      <ChipFilter label="Kind" value={filters.kind} all={`All kinds ${total}`} options={kindOptions} onChange={(kind) => onChange({ ...filters, kind })} />
    </Inline>
  );
};

export { ActivityFilterBar };
