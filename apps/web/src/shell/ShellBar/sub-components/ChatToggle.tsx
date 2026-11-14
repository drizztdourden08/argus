/* @layer renderer-app @kind component */
import { Icon, IconButton } from '@drizztdourden08/tessera';
import { useChatOpen } from '../../OverlordChat';

const ChatToggle = () => {
  const { toggle } = useChatOpen();
  return (
    <IconButton label="Overlord chat" size="sm" variant="ghost" onClick={toggle}>
      <Icon name="message-square" />
    </IconButton>
  );
};

export { ChatToggle };
