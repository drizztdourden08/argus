/* @layer renderer-app @kind component */
import { Flex, ListItemRow, Text } from '@drizztdourden08/tessera';
import type { ListItemColumn } from '@drizztdourden08/tessera';
import { rowLinked } from '../behavior/row-linked';
import type { RecordRowProps } from '../record-rows.type';
import { AssigneeMark } from './AssigneeMark';
import { RecordMenu } from './RecordMenu';
import { RowChips } from './RowChips';
import { RowCode } from './RowCode';
import { RowTitle } from './RowTitle';
import { TypeIcon } from './TypeIcon';

const RecordRow = (props: RecordRowProps) => {
  const { head, title: given, onOpen, chips = [], age, trailing } = props;
  const untitled = given === '';
  const title = untitled ? head.code : given;
  const linked = onOpen !== undefined && rowLinked(title, chips);
  const columns: ListItemColumn[] = [];
  if (chips.length > 0) columns.push({ primary: <RowChips chips={chips} board={head.boardKey} /> });
  if (age) columns.push({ primary: <Text variant="caption" tone="muted">{age}</Text>, align: 'end' });
  const meta = untitled ? undefined : <RowCode code={head.code} board={head.boardKey} linked={linked} onOpen={onOpen} />;
  return (
    <ListItemRow
      icon={<TypeIcon kind={head.kind} />}
      name={<RowTitle title={title} board={head.boardKey} linked={linked} onOpen={onOpen} label={`Open ${head.code}: ${title}`} />}
      meta={meta}
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
