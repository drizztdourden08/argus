/* @layer renderer-app @kind component */
import { Flex } from '@drizztdourden08/tessera';
import { copyText } from '../../../screens/board/shared';
import { ItemEyebrow } from '../../../screens/board/tabs/kanban/sub-components/ItemEyebrow';
import { assigneeOfRecord } from '../behavior/assignee-of-record';
import { recordMenuGroups } from '../behavior/record-menu-groups';
import { RecordHead } from '../sub-components/RecordHead';
import type { RecordDataProps } from './pane.type';

const RecordEyebrow = (props: RecordDataProps) => {
  const { data } = props;
  const { kind, code, doc, boardKey, threads, item } = data;
  const menu = recordMenuGroups(kind.id, code, { copy: () => copyText(code) });
  return (
    <Flex direction="column" gap="xs">
      <RecordHead
        kind={kind.id}
        code={code}
        assignee={assigneeOfRecord(kind, doc ?? {}, boardKey)}
        boardKey={boardKey}
        threads={threads}
        menu={menu}
        menuLabel={`More actions for ${code}`}
        size="sm"
      />
      {item && <ItemEyebrow item={item.item} waiting={item.waiting} />}
    </Flex>
  );
};

export { RecordEyebrow };
