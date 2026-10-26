/* @layer renderer-app @kind component */
import { Flex } from '@drizztdourden08/tessera';
import { useBoardWork } from '../feed-model';
import { OwnerFeed } from '../OwnerFeed';
import { RegisteredStrip } from '../RegisteredStrip';
import { WorkItems } from '../WorkItems';
import type { BoardBodyProps } from './BoardBody.type';

const BoardBody = (props: BoardBodyProps) => {
  const { boardKey } = props;
  const work = useBoardWork(boardKey);
  return (
    <Flex direction="column" gap="lg">
      <RegisteredStrip boardKey={boardKey} />
      {work.ready ? <OwnerFeed boardKey={boardKey} work={work} /> : null}
      {work.ready ? <WorkItems boardKey={boardKey} items={work.items} needs={work.needs} /> : null}
    </Flex>
  );
};

export { BoardBody };
