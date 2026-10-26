/* @layer renderer-app @kind component */
import { Box, Dialog, MarkdownEditor, SegmentedControl, Stack } from '@drizztdourden08/tessera';
import type { KeyboardEvent } from 'react';
import { SEND_KINDS, SEND_MESSAGE } from '../dashboards.constants';
import type { SendDownDialogProps } from '../dashboards.type';
import { OWNER_MAX, sendProblem } from '../model';
import type { SendKind } from '../model';

const SendDownDialog = (props: SendDownDialogProps) => {
  const { send } = props;
  const trimmed = send.text.trim();
  const over = send.text.length > OWNER_MAX ? sendProblem(send.text) : null;
  const submit = (event: KeyboardEvent<HTMLElement>) => {
    if ((event.ctrlKey || event.metaKey) && event.key === 'Enter') {
      event.preventDefault();
      send.send();
    }
  };
  return (
    <Dialog
      open={send.target !== null}
      title={`Send down to ${send.name}`}
      message={SEND_MESSAGE}
      size="md"
      confirmLabel="Send down"
      confirmDisabled={send.busy || trimmed.length === 0 || over !== null}
      onConfirm={send.send}
      onCancel={send.close}
    >
      <Box onKeyDown={submit}>
        <Stack gap="sm">
          <SegmentedControl<SendKind> aria-label="Kind of post" value={send.kind} options={SEND_KINDS} onChange={send.setKind} />
          <MarkdownEditor aria-label="Text to send down" value={send.text} onChange={send.setText} error={over} hint="Ctrl+Enter sends" />
        </Stack>
      </Box>
    </Dialog>
  );
};

export { SendDownDialog };
