/* @layer renderer-app @kind component */
import { Callout } from '@drizztdourden08/tessera';
import { SWITCH_CANCELLED, SWITCH_DONE, SWITCH_FAILED } from '../switch.constants';
import type { SwitchOutcomeProps } from '../switch.type';

const SwitchOutcome = (props: SwitchOutcomeProps) => {
  const { doc } = props.feed;
  if (doc?.phase === 'done') return <Callout tone="success">{SWITCH_DONE}</Callout>;
  if (doc?.phase === 'cancelled') return <Callout tone="info">{SWITCH_CANCELLED}</Callout>;
  if (doc?.phase === 'failed') return <Callout tone="danger">{doc.error ?? SWITCH_FAILED}</Callout>;
  return null;
};

export { SwitchOutcome };
