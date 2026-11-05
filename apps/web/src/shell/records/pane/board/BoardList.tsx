/* @layer renderer-app @kind component */
import { Flex } from '@drizztdourden08/tessera';
import { DrawerSection } from '../../../../screens/board/tabs/kanban/sub-components/DrawerSection';
import { EmptyLine } from '../../../../screens/board/tabs/kanban/sub-components/EmptyLine';
import type { BoardListProps } from './board.type';

const BoardList = (props: BoardListProps) => {
  const { title, empty, count, children } = props;
  return (
    <DrawerSection title={`${title} · ${count}`}>
      {count === 0 && <EmptyLine>{empty}</EmptyLine>}
      <Flex direction="column" gap="xs" role="list" aria-label={title}>
        {children}
      </Flex>
    </DrawerSection>
  );
};

export { BoardList };
