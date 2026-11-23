/* @layer renderer-app @kind component */
import { Callout, Flex, Spinner } from '@drizztdourden08/tessera';
import { useChatSettings } from '../behavior/useChatSettings';
import { useChatUsage } from '../behavior/useChatUsage';
import { SWITCHING_NOTE } from '../OverlordChat.constants';
import type { ChatBarProps } from '../OverlordChat.type';
import { SettingChips } from './SettingChips';
import { UsageRing } from './UsageRing';

const ChatBar = (props: ChatBarProps) => {
  const { working } = props;
  const chat = useChatSettings();
  const usage = useChatUsage(working);
  return (
    <>
      <Flex gap="xs" align="center" justify="end" className="overlord-chat__bar">
        {chat.busy && <Spinner size="sm" label={SWITCHING_NOTE} />}
        <SettingChips chat={chat} />
        <UsageRing usage={usage} />
      </Flex>
      {chat.problem && <Callout tone="danger">{chat.problem}</Callout>}
    </>
  );
};

export { ChatBar };
