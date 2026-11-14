/* @layer renderer-app @kind component */
import { Drawer } from '@drizztdourden08/tessera';
import { useChatOpen } from './behavior/useChatOpen';
import { CHAT_LABEL } from './OverlordChat.constants';
import { ChatBody } from './sub-components/ChatBody';
import './OverlordChat.css';

const OverlordChat = () => {
  const { open, close } = useChatOpen();
  return (
    <Drawer open={open} onClose={close} side="left" look="record" stackNarrow size="wide" label={CHAT_LABEL} title={CHAT_LABEL}>
      {open && <ChatBody />}
    </Drawer>
  );
};

export { OverlordChat };
