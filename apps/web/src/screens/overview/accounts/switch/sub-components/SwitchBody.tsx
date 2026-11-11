/* @layer renderer-app @kind component */
import { Stack, Text, Toggle } from '@drizztdourden08/tessera';
import { isOver, isRunning } from '../model';
import { DESKTOP_CLOSES, HANDOVER_TOGGLE, HANDOVER_TOGGLE_HELP } from '../switch.constants';
import type { SwitchBodyProps } from '../switch.type';
import { SwitchOutcome } from './SwitchOutcome';
import { SwitchRoute } from './SwitchRoute';
import { SwitchSteps } from './SwitchSteps';

const SwitchBody = (props: SwitchBodyProps) => {
  const { target, accounts, feed, actions, handover, onHandover } = props;
  const running = isRunning(feed.doc, target.id);
  const over = isOver(feed.doc, target.id);
  const from = accounts.find((account) => account.active === true);
  return (
    <Stack gap="md">
      <SwitchRoute from={from} to={target} />
      {!running && !over && <Toggle checked={handover} onChange={onHandover} label={HANDOVER_TOGGLE} description={HANDOVER_TOGGLE_HELP} />}
      {(running || over) && <SwitchSteps feed={feed} actions={actions} />}
      {running && <Text variant="caption" tone="muted">{DESKTOP_CLOSES}</Text>}
      {over && <SwitchOutcome feed={feed} />}
    </Stack>
  );
};

export { SwitchBody };
