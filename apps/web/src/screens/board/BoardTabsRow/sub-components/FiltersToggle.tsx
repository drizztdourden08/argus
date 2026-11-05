/* @layer renderer-app @kind component */
import { Badge, Icon, IconButton } from '@drizztdourden08/tessera';
import { FILTERS_LABEL } from '../BoardTabsRow.constants';
import type { FiltersToggleProps } from '../BoardTabsRow.type';

const FiltersToggle = (props: FiltersToggleProps) => {
  const { open, count, onToggle } = props;
  return (
    <Badge variant="number" color="primary" value={count} hideZero label={`${count} active`}>
      <IconButton size="sm" variant="ghost" label={FILTERS_LABEL} active={open} aria-pressed={open} aria-expanded={open} onClick={onToggle}>
        <Icon name="sliders-horizontal" />
      </IconButton>
    </Badge>
  );
};

export { FiltersToggle };
