/* @layer renderer-app @kind component */
import { Select } from '@drizztdourden08/tessera';
import { useNavigate } from 'react-router';
import { OVERVIEW_CHOICE, OVERVIEW_LABEL } from '../overview.constants';
import type { ProjectSwitcherProps } from '../overview.type';

const ProjectSwitcher = (props: ProjectSwitcherProps) => {
  const { cards } = props;
  const navigate = useNavigate();
  const options = [{ value: OVERVIEW_CHOICE, label: OVERVIEW_LABEL }, ...cards.map((card) => ({ value: card.key, label: card.name }))];
  return (
    <Select
      aria-label="Switch project"
      size="sm"
      options={options}
      value={OVERVIEW_CHOICE}
      onChange={(key) => { if (key !== OVERVIEW_CHOICE) void navigate(`/p/${encodeURIComponent(key)}/`); }}
    />
  );
};

export { ProjectSwitcher };
