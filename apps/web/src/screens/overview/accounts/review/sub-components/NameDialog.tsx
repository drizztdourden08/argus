/* @layer renderer-app @kind component */
import { Dialog, TextInput } from '@drizztdourden08/tessera';
import { useState } from 'react';
import type { NameDialogProps } from '../review-props.type';

const NameDialog = (props: NameDialogProps) => {
  const { title, label, initial, confirmLabel, onConfirm, onCancel } = props;
  const [name, setName] = useState(initial);
  const clean = name.trim();
  return (
    <Dialog open title={title} message="" confirmLabel={confirmLabel} confirmDisabled={clean === ''} onConfirm={() => onConfirm(clean)} onCancel={onCancel}>
      <TextInput aria-label={label} placeholder={label} autoComplete="off" value={name} onChange={(event) => setName(event.target.value)} onEnter={(value) => value.trim() !== '' && onConfirm(value.trim())} />
    </Dialog>
  );
};

export { NameDialog };
