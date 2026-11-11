/* @layer renderer-app @kind component */
import { Dialog, Stack } from '@drizztdourden08/tessera';
import { APPLY_NOTE } from '../review.constants';
import type { ApplyDialogProps } from '../review-props.type';
import { TotalList } from './TotalList';

const ApplyDialog = (props: ApplyDialogProps) => {
  const { choiceLines, editLines, count, busy, onConfirm, onCancel } = props;
  return (
    <Dialog
      open
      size="md"
      title={`Apply ${count} changes`}
      message={APPLY_NOTE}
      confirmLabel="Apply"
      confirmDisabled={busy}
      onConfirm={onConfirm}
      onCancel={onCancel}
    >
      <Stack gap="sm">
        <TotalList title="Differences, by choice" lines={choiceLines} />
        <TotalList title="Edits" lines={editLines} />
      </Stack>
    </Dialog>
  );
};

export { ApplyDialog };
