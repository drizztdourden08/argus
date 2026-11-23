/* @layer renderer-app @kind component */
import { Flex, ScrollArea, SegmentedControl } from '@drizztdourden08/tessera';
import { useState } from 'react';
import { useQueue } from '../behavior/useQueue';
import { CHAT_TABS, TABS_LABEL } from '../OverlordChat.constants';
import type { ChatTab } from '../behavior/chat-panels.type';
import { ChatBody } from './ChatBody';
import { QueueTab } from './QueueTab';
import { RulesTab } from './RulesTab';
import { SettingsTab } from './SettingsTab';

const ChatTabs = () => {
  const [tab, setTab] = useState<ChatTab>('chat');
  const queue = useQueue();
  return (
    <Flex direction="column" gap="md" className="overlord-chat__tabs">
      <SegmentedControl label={TABS_LABEL} value={tab} options={CHAT_TABS} onChange={setTab} />
      {tab === 'chat' && <ChatBody feed={queue} />}
      {tab === 'rules' && <ScrollArea className="overlord-chat__pane"><RulesTab /></ScrollArea>}
      {tab === 'queue' && <ScrollArea className="overlord-chat__pane"><QueueTab feed={queue} /></ScrollArea>}
      {tab === 'settings' && <ScrollArea className="overlord-chat__pane"><SettingsTab /></ScrollArea>}
    </Flex>
  );
};

export { ChatTabs };
