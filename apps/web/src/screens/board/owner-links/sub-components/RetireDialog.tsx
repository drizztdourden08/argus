/* @layer renderer-app @kind component */
import { Callout, Dialog, Flex, TextInput } from '@drizztdourden08/tessera';
import { useState } from 'react';
import { retireLauncher } from '../../../../data/posts';
import { useOwnerSend } from '../behavior/useOwnerSend';
import { NOTE_MAX, RETIRE_NOTE } from '../owner-links.constants';
import type { OwnerDialogProps } from '../owner-links.type';

const RetireDialog = (props: OwnerDialogProps) => {
  const { boardKey, ask, onClose } = props;
  const [note, setNote] = useState('');
  const sender = useOwnerSend(onClose);
  return (
    <Dialog
      open
      size="md"
      variant="danger"
      title={`Retire ${ask.label}`}
      message={RETIRE_NOTE}
      confirmLabel="Retire"
      confirmDisabled={sender.busy}
      onConfirm={() => void sender.send(() => retireLauncher(boardKey, ask.id, note.trim()), `${ask.label} is retired.`)}
      onCancel={onClose}
    >
      <Flex direction="column" gap="sm">
        <TextInput aria-label="Why retire it (optional)" placeholder="Why retire it (optional)" maxLength={NOTE_MAX} autoComplete="off" value={note} onChange={(event) => setNote(event.target.value)} />
        {sender.problem && <Callout tone="danger">{sender.problem}</Callout>}
      </Flex>
    </Dialog>
  );
};

export { RetireDialog };
