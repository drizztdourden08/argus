/* @layer renderer-app @kind component */
import { Icon, IconButton } from '@drizztdourden08/tessera';
import { useState } from 'react';
import type { BoardSettingsButtonProps } from './BoardSettings.type';
import { BoardSettingsDialog } from './sub-components/BoardSettingsDialog';

const BoardSettingsButton = (props: BoardSettingsButtonProps) => {
  const [open, setOpen] = useState(false);
  return (
    <>
      <IconButton label="Board settings" variant="ghost" size="sm" onClick={() => setOpen(true)}>
        <Icon name="settings" />
      </IconButton>
      {open && <BoardSettingsDialog boardKey={props.boardKey} onClose={() => setOpen(false)} />}
    </>
  );
};

export { BoardSettingsButton };
