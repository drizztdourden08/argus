/* @layer renderer-app @kind component */
import { Age } from '../../../../shell';
import type { FaultTipProps } from '../RegisteredStrip.type';

const FaultTip = (props: FaultTipProps) => {
  const { fault, main } = props;
  if (fault.reason === 'board-closed') {
    return (
      <>
        Board closed since <Age value={fault.since} />: this thread has no open board tab. The Overlord is
        resolving it with the thread{main ? '.' : ' and it clears itself when the board opens.'}
      </>
    );
  }
  return (
    <>
      Wake failed: {fault.reason.replace(/-/g, ' ')} since <Age value={fault.since} />. Only the Overlord can
      clear this{main ? '.' : '; it has been notified.'}
    </>
  );
};

export { FaultTip };
