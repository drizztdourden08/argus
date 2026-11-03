/* @layer renderer-app @kind component */
import { Flex } from '@drizztdourden08/tessera';
import { DrawerSection } from '../../../../screens/board/tabs/kanban/sub-components/DrawerSection';
import { EmptyLine } from '../../../../screens/board/tabs/kanban/sub-components/EmptyLine';
import { fieldKeys } from '../../behavior/field-keys';
import { RecordLink } from '../../sub-components/RecordLink';
import type { RecordBodyProps } from '../pane.type';

const RecordLinksTab = (props: RecordBodyProps) => {
  const { kind, doc, boardKey } = props.data;
  return (
    <Flex direction="column" gap="lg">
      {kind.links.map((link) => {
        const ids = fieldKeys(doc, link.field);
        return (
          <DrawerSection key={link.field} title={`${link.label} · ${ids.length}`}>
            {ids.length === 0 && <EmptyLine>{`No ${link.label.toLowerCase()} yet.`}</EmptyLine>}
            <Flex gap="sm" wrap role="list" aria-label={link.label}>
              {ids.map((id) => <RecordLink key={id} id={id} board={boardKey} />)}
            </Flex>
          </DrawerSection>
        );
      })}
    </Flex>
  );
};

export { RecordLinksTab };
