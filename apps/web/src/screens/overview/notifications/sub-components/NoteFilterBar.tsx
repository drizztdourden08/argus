/* @layer renderer-app @kind component */
import { Inline, SegmentedControl, Select } from '@drizztdourden08/tessera';
import type { SegmentOption } from '@drizztdourden08/tessera';
import { useProjectName } from '../../projects';
import { NOTIFY_LEVELS } from '../model';
import type { LevelPick } from '../model';
import { ALL_PROJECTS } from '../notifications.constants';
import type { NoteFiltersProps } from '../notifications.type';

const NoteFilterBar = (props: NoteFiltersProps) => {
  const { filters, counts, projects, onChange } = props;
  const nameOf = useProjectName();
  const levels: SegmentOption<LevelPick>[] = [
    { value: 'all', label: `All levels ${counts.all}` },
    ...NOTIFY_LEVELS.map((level) => ({ value: level, label: `${level} ${counts[level]}` })),
  ];
  const options = [
    { value: ALL_PROJECTS, label: 'All projects' },
    ...projects.map(([project, count]) => ({ value: project, label: `${nameOf(project)} ${count}` })),
  ];
  return (
    <Inline gap="md" align="center" wrap>
      <SegmentedControl<LevelPick> aria-label="Level" value={filters.level} options={levels} onChange={(level) => onChange({ ...filters, level })} />
      {projects.length > 0 && (
        <Select
          aria-label="Project"
          options={options}
          value={filters.project || ALL_PROJECTS}
          onChange={(project) => onChange({ ...filters, project: project === ALL_PROJECTS ? '' : project })}
        />
      )}
    </Inline>
  );
};

export { NoteFilterBar };
