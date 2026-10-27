/* @layer renderer-app @kind component */
import { Select } from '@drizztdourden08/tessera';
import { useNavigate } from 'react-router';
import { useProjectList } from './behavior/useProjectList';
import { OVERVIEW_OPTION } from './shell.constants';
import type { ProjectSwitcherProps } from './shell.type';

const ProjectSwitcher = (props: ProjectSwitcherProps) => {
  const { current } = props;
  const navigate = useNavigate();
  const projects = useProjectList();
  if (projects.length === 0) return null;
  const options = [
    OVERVIEW_OPTION,
    ...projects.map((project) => ({ value: project.key, label: project.running > 0 ? `${project.name} (${project.running} running)` : project.name })),
  ];
  return (
    <Select
      size="sm"
      aria-label="Switch project"
      options={options}
      value={current ?? OVERVIEW_OPTION.value}
      onChange={(key) => void navigate(key === OVERVIEW_OPTION.value ? '/' : `/p/${encodeURIComponent(key)}/`)}
    />
  );
};

export { ProjectSwitcher };
