/* @layer renderer-app @kind component */
import { Button, Inline, Stack, Text, Toggle } from '@drizztdourden08/tessera';
import { useState } from 'react';
import { useSwitchActions } from '../behavior/useSwitchActions';
import { CANCEL_LABEL, HANDOVER_TOGGLE, START_LABEL } from '../switch.constants';
import type { SwitchConfirmProps } from '../switch.type';
import './SwitchConfirm.css';

const SwitchConfirm = (props: SwitchConfirmProps) => {
  const { target, onClose } = props;
  const actions = useSwitchActions();
  const [handover, setHandover] = useState(target.rules?.handover !== 'off');
  const start = () => {
    actions.start(target.id, handover);
    onClose();
  };
  return (
    <Stack gap="xs" className="switch-confirm" role="group" aria-label={START_LABEL}>
      <Text weight={600}>{`Switch Desktop to ${target.label ?? target.id}?`}</Text>
      <Toggle size="sm" checked={handover} onChange={setHandover} label={HANDOVER_TOGGLE} />
      <Inline gap="xs">
        <Button size="xs" disabled={actions.busy} onClick={start}>{START_LABEL}</Button>
        <Button size="xs" variant="ghost" onClick={onClose}>{CANCEL_LABEL}</Button>
      </Inline>
    </Stack>
  );
};

export { SwitchConfirm };
