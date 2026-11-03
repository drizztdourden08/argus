/* @layer renderer-app @kind component */
import { Flex, ListItemRow, Pressable, Text } from '@drizztdourden08/tessera';
import type { ListItemColumn } from '@drizztdourden08/tessera';
import { rowLinked } from '../behavior/row-linked';
import type { RecordRowProps } from '../record-rows.type';
import { AssigneeMark } from './AssigneeMark';
import { RecordMenu } from './RecordMenu';
import { RowChips } from './RowChips';
import { RowTitle } from './RowTitle';
import { TypeIcon } from './TypeIcon';

const RecordRow = (props: RecordRowProps) => {
  const { head, title, onOpen, chips = [], age, trailing } = props;
  const linked = onOpen !== undefined && rowLinked(title, chips);
  const columns: ListItemColumn[] = [];
  if (chips.length > 0) columns.push({ primary: <RowChips chips={chips} board={head.boardKey} /> });
  if (age) columns.push({ primary: <Text variant="caption" tone="muted">{age}</Text>, align: 'end' });
  const code = <Text as="span" variant="caption" tone="muted" mono>{head.code}</Text>;
  return (
    <ListItemRow
      icon={<TypeIcon kind={head.kind} />}
      name={<RowTitle title={title} board={head.boardKey} linked={linked} onOpen={onOpen} label={`Open ${head.code}: ${title}`} />}
      meta={linked ? <Pressable onClick={onOpen} title={`Open ${head.code}`}>{code}</Pressable> : code}
      columns={columns}
      action={
        <Flex gap="xs" align="center">
          {trailing}
          <AssigneeMark assignee={head.assignee} boardKey={head.boardKey} threads={head.threads} />
          <RecordMenu groups={head.menu} label={head.menuLabel} />
        </Flex>
      }
      data={{ 'data-record': head.code }}
    />
  );
};

export { RecordRow };
