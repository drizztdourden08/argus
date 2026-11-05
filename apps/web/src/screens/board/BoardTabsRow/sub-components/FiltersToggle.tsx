/* @layer renderer-app @kind component */
import { Badge, Button, Flex } from '@drizztdourden08/tessera';
import { FILTERS_LABEL } from '../BoardTabsRow.constants';
import type { FiltersToggleProps } from '../BoardTabsRow.type';

const FiltersToggle = (props: FiltersToggleProps) => {
  const { open, count, onToggle } = props;
  return (
    <Button size="sm" variant="ghost" active={open} aria-pressed={open} aria-expanded={open} onClick={onToggle}>
      <Flex inline gap="xs" align="center">
        {FILTERS_LABEL}
        {count > 0 && <Badge variant="number" color="primary" value={count} label={`${count} active`} />}
      </Flex>
    </Button>
  );
};

export { FiltersToggle };
