/* @layer renderer-app @kind component */
import { Flex } from '@drizztdourden08/tessera';
import { DrawerSection } from '../../../screens/board/tabs/kanban/sub-components/DrawerSection';
import { ACROSS } from './BlockedSection.constants';
import { CauseRow } from './CauseRow';
import type { BlockedSectionProps } from './pane.type';

const BlockedSection = (props: BlockedSectionProps) => {
  const { item, items } = props.data;
  const causes = (item.blocked ?? []).filter((cause) => !ACROSS.includes(cause.kind));
  const blocks = items.filter((other) => !other.deletedAt && other.blockedBy?.includes(item.id));
  if (causes.length === 0 && blocks.length === 0) return null;
  return (
    <DrawerSection title="Blocked by and blocks">
      <Flex direction="column" gap="xs">
        {causes.map((cause) => (
          <CauseRow key={`${cause.kind}-${cause.id}`} kind={cause.kind} id={cause.id} label={cause.label} />
        ))}
        {blocks.map((other) => (
          <CauseRow key={`blocks-${other.id}`} kind="blocks" id={other.id} label={other.title} />
        ))}
      </Flex>
    </DrawerSection>
  );
};

export { BlockedSection };
