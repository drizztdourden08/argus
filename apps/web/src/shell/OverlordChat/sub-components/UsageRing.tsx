/* @layer renderer-app @kind component */
import { Anchored, Card, IconButton, ProgressRing } from '@drizztdourden08/tessera';
import { contextShare } from '../behavior/context-share';
import { useUsagePopover } from '../behavior/useUsagePopover';
import { USAGE_LABEL } from '../OverlordChat.constants';
import type { UsageRingProps } from '../chat-settings.type';
import { UsagePanel } from './UsagePanel';

const UsageRing = (props: UsageRingProps) => {
  const { usage } = props;
  const { anchor, open, toggle, close } = useUsagePopover(usage.reload);
  return (
    <>
      <IconButton ref={anchor} label={USAGE_LABEL} variant="ghost" size="xs" aria-haspopup="dialog" aria-expanded={open} onClick={toggle}>
        <ProgressRing progress={contextShare(usage.usage)} className="overlord-chat__ring" />
      </IconButton>
      {open && usage.usage && (
        <Anchored anchorRef={anchor} placement="top-end" onDismiss={close} className="overlord-chat__usage" role="dialog" aria-label={USAGE_LABEL}>
          <Card surface="glass">
            <UsagePanel usage={usage.usage} />
          </Card>
        </Anchored>
      )}
    </>
  );
};

export { UsageRing };
