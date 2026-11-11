/* @layer renderer-app @kind component */
import { Button } from '@drizztdourden08/tessera';
import { canCancel, isOver, isRunning } from '../model';
import { CANCEL_LABEL, CLOSE_LABEL, START_LABEL } from '../switch.constants';
import type { SwitchFooterProps } from '../switch.type';

const SwitchFooter = (props: SwitchFooterProps) => {
  const { target, feed, actions, handover, onClose } = props;
  const running = isRunning(feed.doc, target.id);
  const over = isOver(feed.doc, target.id);
  return (
    <>
      {running && <Button size="sm" variant="secondary" disabled={!canCancel(feed.doc) || actions.busy} onClick={actions.cancel}>{CANCEL_LABEL}</Button>}
      {!running && !over && <Button size="sm" disabled={actions.busy || target.rules?.canReceive === false} onClick={() => actions.start(target.id, handover)}>{START_LABEL}</Button>}
      <Button size="sm" variant="ghost" onClick={onClose}>{running || over ? CLOSE_LABEL : CANCEL_LABEL}</Button>
    </>
  );
};

export { SwitchFooter };
