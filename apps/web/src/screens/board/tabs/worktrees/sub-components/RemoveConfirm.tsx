/* @layer renderer-app @kind component */
import { Checkbox, Flex } from '@drizztdourden08/tessera';
import { confirmStale, isSafe } from '../model';
import type { RemoveConfirmProps } from '../worktrees-props.type';
import { ConfirmFooter } from './ConfirmFooter';
import { LostText } from './LostText';
import { SafeText } from './SafeText';

const RemoveConfirm = (props: RemoveConfirmProps) => {
  const { row, panel, controls, removal } = props;
  const safe = isSafe(row);
  const stale = confirmStale(row, panel);
  return (
    <Flex direction="column" gap="sm" role="group" aria-label="Confirm the removal">
      {safe ? <SafeText row={row} /> : <LostText row={row} />}
      {row.branch && (
        <Checkbox
          checked={panel.branch}
          label={`also delete the branch ${row.branch}`}
          onChange={(branch) => controls.patch({ branch })}
        />
      )}
      <ConfirmFooter
        stale={stale}
        error={panel.error}
        label={removal.busy ? 'Removing' : 'Confirm'}
        danger={!safe}
        disabled={removal.busy}
        onConfirm={() => void removal.confirm(row)}
        onCancel={controls.close}
      />
    </Flex>
  );
};

export { RemoveConfirm };
