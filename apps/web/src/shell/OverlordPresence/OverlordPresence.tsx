/* @layer renderer-app @kind component */
import { BrandMark, Pressable, ProgressRing, Tooltip } from '@drizztdourden08/tessera';
import { useChatOpen, useOverlordStatus } from '../OverlordChat';
import { presenceLook } from './behavior/presence-look';
import { useSecondClock } from './behavior/useSecondClock';
import { PRESENCE_LABEL } from './OverlordPresence.constants';
import './OverlordPresence.css';

const OverlordPresence = () => {
  const { toggle } = useChatOpen();
  const status = useOverlordStatus();
  const now = useSecondClock();
  const look = presenceLook({ status: status.data, now });
  return (
    <Tooltip content={look.tip} placement="bottom">
      <Pressable className={`overlord-presence is-${look.activity}`} aria-label={PRESENCE_LABEL} onClick={toggle}>
        {look.progress !== null && <ProgressRing className="overlord-presence__ring" progress={look.progress} />}
        <BrandMark app="argus" size="lg" />
      </Pressable>
    </Tooltip>
  );
};

export { OverlordPresence };
