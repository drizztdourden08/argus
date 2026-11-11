/* @layer renderer-app @kind component */
import { Dialog, Select } from '@drizztdourden08/tessera';
import { useMemo, useState } from 'react';
import { NO_FOLDER, NO_FOLDER_VALUE } from '../review.constants';
import type { MoveDialogProps } from '../review-props.type';

const MoveDialog = (props: MoveDialogProps) => {
  const { account, count, folders, onConfirm, onCancel } = props;
  const [value, setValue] = useState(NO_FOLDER_VALUE);
  const options = useMemo(() => [{ value: NO_FOLDER_VALUE, label: NO_FOLDER }, ...folders.map((name) => ({ value: name, label: name }))], [folders]);
  return (
    <Dialog
      open
      title={count === 1 ? 'Move chat to folder' : `Move ${count} chats to folder`}
      message={`Folders on ${account}.`}
      confirmLabel="Move"
      onConfirm={() => onConfirm(value === NO_FOLDER_VALUE ? null : value)}
      onCancel={onCancel}
    >
      <Select aria-label="Folder" options={options} value={value} onChange={setValue} />
    </Dialog>
  );
};

export { MoveDialog };
