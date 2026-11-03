/* @layer renderer-app @kind component */
import { Flex, Pressable, Tag, Text } from '@drizztdourden08/tessera';
import { assigneeOf, stageLabel } from '../../../screens/board/tabs/kanban/model';
import { itemCode } from '../behavior/item-code';
import { AssigneeMark } from '../sub-components/AssigneeMark';
import { ItemTypeMark } from '../sub-components/ItemTypeMark';
import type { ChildRowProps } from './pane.type';

const ChildRow = (props: ChildRowProps) => {
  const { child, data } = props;
  const code = itemCode(child);
  const criteria = child.ac ?? [];
  const met = criteria.filter((criterion) => criterion.done).length;
  return (
    <Flex gap="sm" align="center" wrap role="listitem">
      <ItemTypeMark item={child} />
      <Pressable aria-label={`Open ${code}: ${child.title}`} onClick={() => data.open(child.id)}>
        <Flex gap="xs" align="center">
          <Text as="span" variant="caption" weight={600} mono>{code}</Text>
          <Text as="span" smartTimes>{child.title}</Text>
        </Flex>
      </Pressable>
      <Tag>{stageLabel(child, 'assignee')}</Tag>
      <AssigneeMark assignee={assigneeOf(child)} boardKey={data.boardKey} threads={data.threads} />
      {criteria.length > 0 && <Text as="span" variant="caption" tone="muted" title={`${met} of ${criteria.length} criteria met`}>{`${met}/${criteria.length}`}</Text>}
    </Flex>
  );
};

export { ChildRow };
