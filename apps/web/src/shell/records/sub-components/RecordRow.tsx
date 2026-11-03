/* @layer renderer-app @kind component */
import { Flex, ListItemRow, Tag, Text } from '@drizztdourden08/tessera';
import type { ListItemColumn } from '@drizztdourden08/tessera';
import { ROW_CHIPS_MAX } from '../records.constants';
import type { RecordRowProps } from '../records.type';
import { AssigneeMark } from './AssigneeMark';
import { RecordMenu } from './RecordMenu';
import { TypeIcon } from './TypeIcon';

const RecordRow = (props: RecordRowProps) => {
  const { head, title, onOpen, chips = [], age, trailing } = props;
  const columns: ListItemColumn[] = [];
  if (chips.length > 0) {
    columns.push({
      primary: (
        <Flex gap="xs" align="center">
          {chips.slice(0, ROW_CHIPS_MAX).map((chip) => <Tag key={chip.id} title={chip.title}>{chip.text}</Tag>)}
        </Flex>
      ),
    });
  }
  if (age) columns.push({ primary: <Text variant="caption" tone="muted">{age}</Text>, align: 'end' });
  return (
    <ListItemRow
      icon={<TypeIcon kind={head.kind} />}
      name={<Text clamp={1} smartTimes>{title}</Text>}
      meta={<Text as="span" variant="caption" tone="muted" mono>{head.code}</Text>}
      columns={columns}
      action={
        <Flex gap="xs" align="center">
          {trailing}
          <AssigneeMark assignee={head.assignee} boardKey={head.boardKey} threads={head.threads} />
          <RecordMenu groups={head.menu} label={head.menuLabel} />
        </Flex>
      }
      onClick={onOpen}
      data={{ 'data-record': head.code }}
    />
  );
};

export { RecordRow };
