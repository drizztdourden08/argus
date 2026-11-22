/* @layer renderer-app @kind component */
import { Drawer } from '@drizztdourden08/tessera';
import { useChatOpen } from './behavior/useChatOpen';
import { CHAT_LABEL } from './OverlordChat.constants';
import { ChatControls } from './sub-components/ChatControls';
import { ChatTabs } from './sub-components/ChatTabs';
import './OverlordChat.css';

const OverlordChat = () => {
  const { open, close } = useChatOpen();
  return (
    <Drawer
      open={open}
      onClose={close}
      side="left"
      look="record"
      stackNarrow
      size="wide"
      label={CHAT_LABEL}
      title={CHAT_LABEL}
      headerActions={<ChatControls />}
    >
      {open && <ChatTabs />}
    </Drawer>
  );
};

export { OverlordChat };
