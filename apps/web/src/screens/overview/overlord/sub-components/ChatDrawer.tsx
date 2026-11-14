/* @layer renderer-app @kind component */
import { Drawer, Flex } from '@drizztdourden08/tessera';
import type { ChatDrawerProps } from '../overlord.type';
import { ChatBox } from './ChatBox';
import { TranscriptView } from './TranscriptView';

const ChatDrawer = (props: ChatDrawerProps) => {
  const { open, rows, problem, running, action, onSend, onClose } = props;
  return (
    <Drawer open={open} onClose={onClose} side="left" look="record" stackNarrow size="wide" label="Overlord chat" title="Overlord chat">
      <Flex direction="column" gap="lg">
        <TranscriptView rows={rows} problem={problem} />
        <ChatBox running={running} action={action} onSend={onSend} />
      </Flex>
    </Drawer>
  );
};

export { ChatDrawer };
