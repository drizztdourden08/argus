/* @layer renderer-app @kind component */
import { SegmentedControl } from '@drizztdourden08/tessera';
import type { SegmentOption } from '@drizztdourden08/tessera';
import { LEVELS } from '../model';
import type { LevelFilter as Level } from '../model';
import type { LevelFilterProps } from '../activity.type';

const LevelFilter = (props: LevelFilterProps) => {
  const { value, counts, onChange } = props;
  const options: SegmentOption<Level>[] = [
    { value: 'all', label: `All levels ${counts.all}` },
    ...LEVELS.map((level) => ({ value: level, label: `${level} ${counts[level]}` })),
  ];
  return <SegmentedControl<Level> aria-label="Level" value={value} options={options} onChange={onChange} />;
};

export { LevelFilter };
