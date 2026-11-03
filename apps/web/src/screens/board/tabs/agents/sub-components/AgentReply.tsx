/* @layer renderer-app @kind component */
import { Text } from '@drizztdourden08/tessera';
import { BoardMarkdown } from '../../../../../shell';
import { DrawerSection } from '../../kanban/sub-components/DrawerSection';
import type { AgentReplyProps } from '../row-props.type';

const AgentReply = (props: AgentReplyProps) => {
  const { title, text, empty } = props;
  return (
    <DrawerSection title={title}>
      {text ? <BoardMarkdown>{text}</BoardMarkdown> : <Text tone="muted">{empty}</Text>}
    </DrawerSection>
  );
};

export { AgentReply };
