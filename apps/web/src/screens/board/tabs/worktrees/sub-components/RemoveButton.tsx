/* @layer renderer-app @kind component */
import { Button } from '@drizztdourden08/tessera';
import { canRemove, removeLook } from '../model';
import type { RemoveButtonProps } from '../worktrees-props.type';

const RemoveButton = (props: RemoveButtonProps) => {
  const { row, context } = props;
  if (!canRemove(row)) return null;
  const { panel, busy, onAsk } = context;
  const look = removeLook(row);
  return (
    <Button
      size="xs"
      variant={look.danger ? 'danger' : 'tertiary'}
      title={look.tip}
      aria-expanded={panel?.id === row.id && panel.kind === 'remove'}
      disabled={busy && panel?.id === row.id}
      onClick={() => onAsk(row, 'remove')}
    >
      {look.label}
    </Button>
  );
};

export { RemoveButton };
