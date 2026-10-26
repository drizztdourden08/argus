/* @layer renderer-app @kind component */
import { Inline, SegmentedControl } from '@drizztdourden08/tessera';
import type { SegmentOption } from '@drizztdourden08/tessera';
import { useProjectName } from '../../projects';
import { ChipFilter } from '../../sub-components/ChipFilter';
import { NOTIFY_LEVELS } from '../model';
import type { LevelPick } from '../model';
import type { NoteFiltersProps } from '../notifications.type';

const NoteFilterBar = (props: NoteFiltersProps) => {
  const { filters, counts, projects, onChange } = props;
  const nameOf = useProjectName();
  const levels: SegmentOption<LevelPick>[] = [
    { value: 'all', label: `All levels ${counts.all}` },
    ...NOTIFY_LEVELS.map((level) => ({ value: level, label: `${level} ${counts[level]}` })),
  ];
  const options = projects.map(([project, count]) => ({ value: project, label: `${nameOf(project)} ${count}` }));
  return (
    <Inline gap="md" align="center" wrap>
      <SegmentedControl<LevelPick> aria-label="Level" value={filters.level} options={levels} onChange={(level) => onChange({ ...filters, level })} />
      {projects.length > 0 && (
        <ChipFilter label="Project" value={filters.project} all={`All projects ${counts.all}`} options={options} onChange={(project) => onChange({ ...filters, project })} />
      )}
    </Inline>
  );
};

export { NoteFilterBar };
