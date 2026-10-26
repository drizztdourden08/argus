/* @layer renderer-app @kind component */
import { Inline } from '@drizztdourden08/tessera';
import { ChipFilter } from '../../sub-components/ChipFilter';
import type { ActivityFilterBarProps } from '../activity.type';
import { LEVELS } from '../model';

const ActivityFilterBar = (props: ActivityFilterBarProps) => {
  const { filters, levels, kinds, total, onChange } = props;
  const levelOptions = LEVELS.map((level) => ({ value: level, label: `${level} ${levels[level]}` }));
  const kindOptions = kinds.map(([kind, count]) => ({ value: kind, label: `${kind} ${count}` }));
  return (
    <Inline gap="md" align="center" wrap>
      <ChipFilter
        label="Level"
        value={filters.level === 'all' ? '' : filters.level}
        all={`All levels ${levels.all}`}
        options={levelOptions}
        onChange={(level) => onChange({ ...filters, level: LEVELS.find((known) => known === level) ?? 'all' })}
      />
      <ChipFilter label="Kind" value={filters.kind} all={`All kinds ${total}`} options={kindOptions} onChange={(kind) => onChange({ ...filters, kind })} />
    </Inline>
  );
};

export { ActivityFilterBar };
