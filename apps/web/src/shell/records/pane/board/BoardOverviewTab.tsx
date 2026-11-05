/* @layer renderer-app @kind component */
import { Flex, Text } from '@drizztdourden08/tessera';
import { DrawerSection } from '../../../../screens/board/tabs/kanban/sub-components/DrawerSection';
import { RecordLink } from '../../sub-components/RecordLink';
import { BoardList } from './BoardList';
import { BoardRow } from './BoardRow';
import { BoardTextEditor } from './BoardTextEditor';
import type { BoardTabProps } from './board.type';
import { boardFacts } from './model/board-facts';

const BoardOverviewTab = (props: BoardTabProps) => {
  const { data } = props;
  const waiting = data.card?.ownerItems ?? [];
  const attention = data.card?.attention ?? [];
  return (
    <Flex direction="column" gap="lg">
      <DrawerSection title="Goal">
        <BoardTextEditor record={data.record} field="goal" />
      </DrawerSection>
      <DrawerSection title="Note">
        <BoardTextEditor record={data.record} field="note" />
      </DrawerSection>
      <DrawerSection title="Facts">
        <Flex direction="column" gap="xs" role="list" aria-label="Facts">
          {boardFacts(data).map((fact) => (
            <BoardRow key={fact.label}>
              <Text variant="caption" tone="muted">{fact.label}</Text>
              <Text as="span">{fact.text}</Text>
            </BoardRow>
          ))}
        </Flex>
      </DrawerSection>
      <BoardList title="Waiting on you" empty="Nothing waits on you here." count={waiting.length}>
        {waiting.map((owner) => (
          <BoardRow key={owner.id}>
            <RecordLink id={owner.id} board={data.record.key} />
            <Text as="span">{owner.title}</Text>
          </BoardRow>
        ))}
      </BoardList>
      <BoardList title="Attention" empty="Nothing needs attention." count={attention.length}>
        {attention.map((line) => (
          <BoardRow key={line}>
            <Text as="span">{line}</Text>
          </BoardRow>
        ))}
      </BoardList>
    </Flex>
  );
};

export { BoardOverviewTab };
