/* @layer renderer-app @kind component */
import { Status } from '@drizztdourden08/tessera';
import { atLimitText, RECEIVE_STATUSES } from '../accounts.constants';
import type { ReceiveLineProps } from '../cards.type';

const ReceiveLine = (props: ReceiveLineProps) => {
  const { state } = props;
  if (state.full) return <Status tone="danger" size="sm" dot>{atLimitText(state.until)}</Status>;
  return <Status map={RECEIVE_STATUSES} value={state.receive ? 'open' : 'locked'} size="sm" dot />;
};

export { ReceiveLine };
