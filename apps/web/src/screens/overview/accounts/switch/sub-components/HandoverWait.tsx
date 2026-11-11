/* @layer renderer-app @kind component */
import { Button, Inline, Text } from '@drizztdourden08/tessera';
import { readyCount } from '../model';
import { ANYWAY_LABEL } from '../switch.constants';
import type { SwitchStepsProps } from '../switch.type';

const HandoverWait = (props: SwitchStepsProps) => {
  const { feed, actions } = props;
  const { ready, total } = readyCount(feed.handovers);
  return (
    <Inline gap="sm" align="center" wrap>
      <Text variant="caption" tone="muted">{`${ready} of ${total} ready`}</Text>
      <Button size="sm" variant="secondary" disabled={actions.busy} onClick={actions.anyway}>{ANYWAY_LABEL}</Button>
    </Inline>
  );
};

export { HandoverWait };
