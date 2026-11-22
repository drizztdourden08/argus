/* @layer renderer-app @kind component */
import { Flex, SegmentedControl } from '@drizztdourden08/tessera';
import { useState } from 'react';
import { CHAT_TABS, TABS_LABEL } from '../OverlordChat.constants';
import type { ChatTab } from '../OverlordChat.type';
import { ChatBody } from './ChatBody';
import { QueueTab } from './QueueTab';
import { RulesTab } from './RulesTab';

const ChatTabs = () => {
  const [tab, setTab] = useState<ChatTab>('chat');
  return (
    <Flex direction="column" gap="lg">
      <SegmentedControl label={TABS_LABEL} value={tab} options={CHAT_TABS} onChange={setTab} />
      {tab === 'chat' && <ChatBody />}
      {tab === 'rules' && <RulesTab />}
      {tab === 'queue' && <QueueTab />}
    </Flex>
  );
};

export { ChatTabs };
