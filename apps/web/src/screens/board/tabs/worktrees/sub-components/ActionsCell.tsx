/* @layer renderer-app @kind component */
import { Button, Flex } from '@drizztdourden08/tessera';
import { threadOf } from '../model';
import type { PanelKind } from '../model';
import { ASK_BUTTONS } from '../worktrees.constants';
import type { CellProps } from '../worktrees.type';
import { RemoveButton } from './RemoveButton';

const ActionsCell = (props: CellProps) => {
  const { row, context } = props;
  if (row.status === 'deleting') return null;
  const { panel, busy, meta, onAsk } = context;
  const thread = threadOf(row, meta) || 'the thread';
  const expanded = (kind: PanelKind): boolean => panel?.id === row.id && panel.kind === kind;
  return (
    <Flex gap="xs" wrap>
      {ASK_BUTTONS.map((ask) => (
        <Button
          key={ask.kind}
          size="xs"
          title={ask.tip(thread)}
          aria-expanded={expanded(ask.kind)}
          disabled={busy && panel?.id === row.id}
          onClick={() => onAsk(row, ask.kind)}
        >
          {ask.label}
        </Button>
      ))}
      <RemoveButton row={row} context={context} />
    </Flex>
  );
};

export { ActionsCell };
