/* @layer renderer-app @kind component */
import { Flex, Text } from '@drizztdourden08/tessera';
import { aboutSize, daysText } from '../model';
import type { BatchConfirmProps } from '../worktrees-props.type';
import { ConfirmFooter } from './ConfirmFooter';

const LeftoverBatchConfirm = (props: BatchConfirmProps) => {
  const { meta, batch } = props;
  const old = meta?.leftoverOld;
  if (!batch.state || !old || old.count === 0) return null;
  const folders = `${old.count} ${old.count === 1 ? 'folder' : 'folders'}`;
  const leftovers = `${old.count} leftover ${old.count === 1 ? 'folder' : 'folders'}`;
  const days = daysText(old.days);
  return (
    <Flex direction="column" gap="sm" role="group" aria-label="Confirm deleting the old leftover folders">
      <Text>
        <Text.B>Delete {leftovers} older than {days}?</Text.B> Together {aboutSize(old.sizeMB)} (node_modules is not counted). Argus checks each one again (no .git, not listed by git, no nested repository, no running process, nothing changed for {days}), renames it and deletes them one after another in the background. <Text.B>No backup is possible: git does not know these folders.</Text.B>
      </Text>
      <ConfirmFooter
        stale={false}
        error={batch.state.error}
        label={batch.busy ? 'Queuing' : `Delete ${folders}`}
        danger
        disabled={batch.busy}
        onConfirm={() => void batch.confirm()}
        onCancel={batch.cancel}
      />
    </Flex>
  );
};

export { LeftoverBatchConfirm };
