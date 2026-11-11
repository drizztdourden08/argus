/* @layer renderer-app @kind component */
import { Button, Callout, DialogShell, Stack, Text, Toggle } from '@drizztdourden08/tessera';
import { useState } from 'react';
import { useSwitchActions } from '../behavior/useSwitchActions';
import { useSwitchFeed } from '../behavior/useSwitchFeed';
import { canCancel, isOver, isRunning } from '../model';
import {
  CANCEL_LABEL, CLAUDE_CLOSES, CLOSE_LABEL, DIALOG_TITLE, HANDOVER_TOGGLE, HANDOVER_TOGGLE_HELP, START_LABEL, SWITCH_CANCELLED, SWITCH_DONE,
  SWITCH_FAILED,
} from '../switch.constants';
import type { SwitchDialogProps } from '../switch.type';
import { SwitchRoute } from './SwitchRoute';
import { SwitchSteps } from './SwitchSteps';

const SwitchDialog = (props: SwitchDialogProps) => {
  const { target, accounts, onClose } = props;
  const feed = useSwitchFeed();
  const actions = useSwitchActions();
  const [handover, setHandover] = useState(target.rules?.handover !== 'off');
  const running = isRunning(feed.doc, target.id);
  const over = isOver(feed.doc, target.id);
  const phase = feed.doc?.phase;
  const from = accounts.find((account) => account.active === true);
  const footer = (
    <>
      {running && <Button size="sm" variant="secondary" disabled={!canCancel(feed.doc) || actions.busy} onClick={actions.cancel}>{CANCEL_LABEL}</Button>}
      {!running && !over && <Button size="sm" disabled={actions.busy || target.rules?.canReceive === false} onClick={() => actions.start(target.id, handover)}>{START_LABEL}</Button>}
      <Button size="sm" variant="ghost" onClick={onClose}>{running || over ? CLOSE_LABEL : CANCEL_LABEL}</Button>
    </>
  );
  return (
    <DialogShell open size="md" title={`${DIALOG_TITLE} ${target.label ?? target.id}`} onClose={onClose} actions={footer}>
      <Stack gap="md">
        <SwitchRoute from={from} to={target} />
        {!running && !over && <Toggle checked={handover} onChange={setHandover} label={HANDOVER_TOGGLE} description={HANDOVER_TOGGLE_HELP} />}
        {(running || over) && <SwitchSteps feed={feed} actions={actions} />}
        {running && <Text variant="caption" tone="muted">{CLAUDE_CLOSES}</Text>}
        {over && phase === 'done' && <Callout tone="success">{SWITCH_DONE}</Callout>}
        {over && phase === 'cancelled' && <Callout tone="info">{SWITCH_CANCELLED}</Callout>}
        {over && phase === 'failed' && <Callout tone="danger">{feed.doc?.error ?? SWITCH_FAILED}</Callout>}
      </Stack>
    </DialogShell>
  );
};

export { SwitchDialog };
