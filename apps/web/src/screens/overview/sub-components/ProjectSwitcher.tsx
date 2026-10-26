/* @layer renderer-app @kind component */
import { Select } from '@drizztdourden08/tessera';
import { useNavigate } from 'react-router';
import type { ProjectSwitcherProps } from '../overview.type';

const ProjectSwitcher = (props: ProjectSwitcherProps) => {
  const { cards } = props;
  const navigate = useNavigate();
  const options = cards.map((card) => ({ value: card.key, label: card.name }));
  return (
    <Select
      aria-label="Switch project"
      placeholder="Switch project"
      options={options}
      value=""
      onChange={(key) => void navigate(`/p/${encodeURIComponent(key)}/`)}
    />
  );
};

export { ProjectSwitcher };
