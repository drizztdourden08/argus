/* @layer renderer-app @kind component */
import { DialogShell } from '@drizztdourden08/tessera';
import { useState } from 'react';
import { useSwitchActions } from '../behavior/useSwitchActions';
import { useSwitchFeed } from '../behavior/useSwitchFeed';
import { DIALOG_TITLE } from '../switch.constants';
import type { SwitchDialogProps } from '../switch.type';
import { SwitchBody } from './SwitchBody';
import { SwitchFooter } from './SwitchFooter';

const SwitchDialog = (props: SwitchDialogProps) => {
  const { target, accounts, onClose } = props;
  const feed = useSwitchFeed();
  const actions = useSwitchActions();
  const [handover, setHandover] = useState(target.rules?.handover !== 'off');
  const footer = <SwitchFooter target={target} feed={feed} actions={actions} handover={handover} onClose={onClose} />;
  return (
    <DialogShell open size="md" title={`${DIALOG_TITLE} ${target.label ?? target.id}`} onClose={onClose} actions={footer}>
      <SwitchBody target={target} accounts={accounts} feed={feed} actions={actions} handover={handover} onHandover={setHandover} />
    </DialogShell>
  );
};

export { SwitchDialog };
