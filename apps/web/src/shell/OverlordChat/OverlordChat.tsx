/* @layer renderer-app @kind component */
import { Box, Drawer } from '@drizztdourden08/tessera';
import { useChatOpen } from './behavior/useChatOpen';
import { CHAT_LABEL } from './OverlordChat.constants';
import { ChatControls } from './sub-components/ChatControls';
import { ChatTabs } from './sub-components/ChatTabs';
import { OverlordFeedProvider } from './sub-components/OverlordFeedProvider';
import './OverlordChat.css';

const OverlordChat = () => {
  const { open, close } = useChatOpen();
  return (
    <OverlordFeedProvider>
      <Drawer
        open={open}
        onClose={close}
        side="left"
        look="split"
        stackNarrow
        size="wide"
        label={CHAT_LABEL}
        title={<Box className="overlord-chat__mask-gap" aria-hidden />}
        headerActions={<ChatControls />}
      >
        {open && <ChatTabs />}
      </Drawer>
    </OverlordFeedProvider>
  );
};

export { OverlordChat };
