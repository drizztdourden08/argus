/* @layer renderer-app @kind component */
import { Button } from '@drizztdourden08/tessera';
import type { ToolToggleProps } from '../BoardTabsRow.type';

const ToolToggle = (props: ToolToggleProps) => {
  const { label, pressed, count, onToggle } = props;
  return (
    <Button size="sm" variant="ghost" active={pressed} aria-pressed={pressed} onClick={onToggle}>
      {count === undefined ? label : `${label} (${count})`}
    </Button>
  );
};

export { ToolToggle };
