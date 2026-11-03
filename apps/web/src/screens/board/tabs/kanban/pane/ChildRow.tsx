/* @layer renderer-app @kind component */
import { Flex, Pressable, Tag, Text } from '@drizztdourden08/tessera';
import { stageLabel } from '../model';
import { AssigneeMark } from '../sub-components/AssigneeMark';
import { ItemTypeMark } from '../sub-components/ItemTypeMark';
import type { ChildRowProps } from './pane.type';

const ChildRow = (props: ChildRowProps) => {
  const { child, data } = props;
  const criteria = child.ac ?? [];
  const met = criteria.filter((criterion) => criterion.done).length;
  return (
    <Flex gap="sm" align="center" wrap role="listitem">
      <ItemTypeMark item={child} />
      <Pressable aria-label={`Open ${child.id}: ${child.title}`} onClick={() => data.open(child.id)}>
        <Flex gap="xs" align="center">
          <Text as="span" variant="caption" weight={600} mono>{child.id}</Text>
          <Text as="span" smartTimes>{child.title}</Text>
        </Flex>
      </Pressable>
      <Tag>{stageLabel(child, 'assignee')}</Tag>
      <AssigneeMark item={child} boardKey={data.boardKey} threads={data.threads} />
      {criteria.length > 0 && <Text as="span" variant="caption" tone="muted" title={`${met} of ${criteria.length} criteria met`}>{`${met}/${criteria.length}`}</Text>}
    </Flex>
  );
};

export { ChildRow };
