/* @layer renderer-app @kind component */
import { Inline, SegmentedControl, Select } from '@drizztdourden08/tessera';
import type { SegmentOption } from '@drizztdourden08/tessera';
import { ALL_KINDS } from '../activity.constants';
import type { ActivityFilterBarProps } from '../activity.type';
import { LEVELS } from '../model';
import type { LevelFilter } from '../model';

const ActivityFilterBar = (props: ActivityFilterBarProps) => {
  const { filters, levels, kinds, total, onChange } = props;
  const levelOptions: SegmentOption<LevelFilter>[] = [
    { value: 'all', label: `All levels ${levels.all}` },
    ...LEVELS.map((level) => ({ value: level, label: `${level} ${levels[level]}` })),
  ];
  const kindOptions = [
    { value: ALL_KINDS, label: `All kinds ${total}` },
    ...kinds.map(([kind, count]) => ({ value: kind, label: `${kind} ${count}` })),
  ];
  return (
    <Inline gap="md" align="center" wrap>
      <SegmentedControl<LevelFilter> aria-label="Level" value={filters.level} options={levelOptions} onChange={(level) => onChange({ ...filters, level })} />
      <Select
        aria-label="Kind"
        options={kindOptions}
        value={filters.kind || ALL_KINDS}
        onChange={(kind) => onChange({ ...filters, kind: kind === ALL_KINDS ? '' : kind })}
      />
    </Inline>
  );
};

export { ActivityFilterBar };
