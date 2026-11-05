/* @layer renderer-app @kind component */
import { Flex, Text } from '@drizztdourden08/tessera';
import { BoardList } from './BoardList';
import { BoardRow } from './BoardRow';
import type { BoardTabProps } from './board.type';

const BoardBuildsTab = (props: BoardTabProps) => {
  const { card, launchers } = props.data;
  const live = launchers.filter((launcher) => !launcher.retiredAt);
  const run = card?.master?.lastRun;
  return (
    <Flex direction="column" gap="lg">
      <BoardList title="Master" empty="This board has no master." count={card?.master ? 1 : 0}>
        {card?.master && (
          <BoardRow>
            <Text as="span" weight={600}>{card.master.title ?? card.master.name}</Text>
            <Text as="span" tone="muted">{run ? `last run ${run.status}` : 'never run'}</Text>
          </BoardRow>
        )}
      </BoardList>
      <BoardList title="Launchers" empty="No launchers." count={live.length}>
        {live.map((launcher) => (
          <BoardRow key={launcher.id}>
            <Text as="span" weight={600}>{launcher.title ?? launcher.name}</Text>
            <Text as="span" tone="muted" mono>{launcher.cmd ?? ''}</Text>
          </BoardRow>
        ))}
      </BoardList>
    </Flex>
  );
};

export { BoardBuildsTab };
