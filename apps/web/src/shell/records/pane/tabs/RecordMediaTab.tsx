/* @layer renderer-app @kind component */
import { EntryImages } from '../../../../screens/board/OwnerFeed';
import { DrawerSection } from '../../../../screens/board/tabs/kanban/sub-components/DrawerSection';
import { EmptyLine } from '../../../../screens/board/tabs/kanban/sub-components/EmptyLine';
import type { RecordBodyProps } from '../pane.type';

const RecordMediaTab = (props: RecordBodyProps) => {
  const { code, entry } = props.data;
  return (
    <DrawerSection title="Media">
      {entry ? <EntryImages entry={entry.entry} /> : <EmptyLine>{`No image on ${code} yet.`}</EmptyLine>}
    </DrawerSection>
  );
};

export { RecordMediaTab };
