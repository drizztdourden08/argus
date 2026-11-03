/* @layer renderer-app @kind component */
import { Flex } from '@drizztdourden08/tessera';
import { AddChild } from '../AddChild';
import { ChildRow } from '../ChildRow';
import { childrenOf } from '../model/children-of';
import type { PaneBodyProps } from '../pane.type';
import { DrawerSection } from '../../../../screens/board/tabs/kanban/sub-components/DrawerSection';
import { EmptyLine } from '../../../../screens/board/tabs/kanban/sub-components/EmptyLine';

const ChildrenTab = (props: PaneBodyProps) => {
  const { data } = props;
  const children = childrenOf(data.items, data.item.id);
  return (
    <DrawerSection title={`Children · ${children.length}`}>
      {children.length === 0 && <EmptyLine>No child on this item.</EmptyLine>}
      <Flex direction="column" gap="xs" role="list" aria-label="Children">
        {children.map((child) => (
          <ChildRow key={child.id} child={child} data={data} />
        ))}
      </Flex>
      <AddChild data={data} />
    </DrawerSection>
  );
};

export { ChildrenTab };
