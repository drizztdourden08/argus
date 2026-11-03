/* @layer renderer-app @kind component */
import { Flex } from '@drizztdourden08/tessera';
import { DrawerSection } from '../../../../screens/board/tabs/kanban/sub-components/DrawerSection';
import { EmptyLine } from '../../../../screens/board/tabs/kanban/sub-components/EmptyLine';
import type { TrailSectionProps } from '../pane.type';
import { TrailRow } from './TrailRow';
import { useTrail } from './useTrail';

const TrailSection = (props: TrailSectionProps) => {
  const { boardKey, item, now, open } = props.data;
  const tree = useTrail(boardKey, item);
  const held = (item.blocked?.length ?? 0) > 0;
  const title = tree ? `Dependency trail · ${tree.depth} deep, ${tree.open} open` : 'Dependency trail';
  return (
    <DrawerSection title={title}>
      {tree && tree.root.children.length > 0 ? (
        <Flex direction="column" role="list" aria-label="Dependency trail">
          <TrailRow node={tree.root} boardKey={boardKey} now={now} open={open} />
        </Flex>
      ) : (
        <EmptyLine>{held ? 'No trail to show yet.' : 'Nothing holds this item back.'}</EmptyLine>
      )}
    </DrawerSection>
  );
};

export { TrailSection };
