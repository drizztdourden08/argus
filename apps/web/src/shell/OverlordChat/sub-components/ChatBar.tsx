/* @layer renderer-app @kind component */
import { Spinner } from '@drizztdourden08/tessera';
import { SWITCHING_NOTE } from '../OverlordChat.constants';
import type { ChatBarProps } from '../behavior/chat-settings.type';
import { SettingChips } from './SettingChips';
import { UsageRing } from './UsageRing';

const ChatBar = (props: ChatBarProps) => {
  const { chat, usage } = props;
  return (
    <>
      <SettingChips chat={chat} />
      <UsageRing usage={usage} />
      {chat.busy && <Spinner size="sm" label={SWITCHING_NOTE} />}
    </>
  );
};

export { ChatBar };
