/* @layer renderer-app @kind component */
import { Callout, Dialog, Flex, Select, TextInput } from '@drizztdourden08/tessera';
import { useState } from 'react';
import { keepWorktree } from '../../../../data/posts';
import { useOwnerSend } from '../behavior/useOwnerSend';
import { DEFAULT_KEEP_DAYS, KEEP_DAYS, KEEP_NOTE, NOTE_MAX } from '../owner-links.constants';
import type { OwnerDialogProps } from '../owner-links.type';

const KeepDialog = (props: OwnerDialogProps) => {
  const { boardKey, ask, onClose } = props;
  const [note, setNote] = useState('');
  const [days, setDays] = useState(DEFAULT_KEEP_DAYS);
  const sender = useOwnerSend(onClose);
  const post = () => keepWorktree(boardKey, ask.id, note.trim(), Number(days));
  return (
    <Dialog
      open
      size="md"
      title={`Keep ${ask.label}`}
      message={KEEP_NOTE}
      confirmLabel="Keep"
      confirmDisabled={sender.busy || note.trim() === ''}
      onConfirm={() => void sender.send(post, `${ask.label} is kept for ${days} days.`)}
      onCancel={onClose}
    >
      <Flex direction="column" gap="sm">
        <TextInput aria-label="Why keep it" placeholder="Why keep it" maxLength={NOTE_MAX} autoComplete="off" value={note} onChange={(event) => setNote(event.target.value)} />
        <Select aria-label="Keep for" options={KEEP_DAYS} value={days} onChange={setDays} />
        {sender.problem && <Callout tone="danger">{sender.problem}</Callout>}
      </Flex>
    </Dialog>
  );
};

export { KeepDialog };
