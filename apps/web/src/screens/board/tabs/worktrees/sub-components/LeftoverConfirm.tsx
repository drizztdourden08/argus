/* @layer renderer-app @kind component */
import { Callout, Flex } from '@drizztdourden08/tessera';
import { confirmStale, daysText } from '../model';
import type { LeftoverConfirmProps } from '../worktrees-props.type';
import { ConfirmFooter } from './ConfirmFooter';
import { LeftoverText } from './LeftoverText';

const LeftoverConfirm = (props: LeftoverConfirmProps) => {
  const { row, panel, limit, controls, removal } = props;
  const young = panel.seen.young;
  const stale = confirmStale(row, panel);
  const idle = young ? 'Delete anyway' : 'Delete leftover';
  return (
    <Flex direction="column" gap="sm" role="group" aria-label="Confirm deleting the leftover folder">
      <LeftoverText row={row} />
      {young && (
        <Callout tone="warning">
          It changed less than {daysText(limit)} ago: something may still be writing there. A thread would refuse it; confirm only if you are sure it is dead.
        </Callout>
      )}
      <ConfirmFooter
        stale={stale}
        error={panel.error}
        label={removal.busy ? 'Deleting' : idle}
        danger
        disabled={removal.busy}
        onConfirm={() => void removal.confirm(row)}
        onCancel={controls.close}
      />
    </Flex>
  );
};

export { LeftoverConfirm };
