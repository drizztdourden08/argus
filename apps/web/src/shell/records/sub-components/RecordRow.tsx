/* @layer renderer-app @kind component */
import { Pressable, Text } from '@drizztdourden08/tessera';
import { rowLinked } from '../behavior/row-linked';
import type { RecordRowProps } from '../record-rows.type';
import { AssigneeMark } from './AssigneeMark';
import { RecordFrame } from './RecordFrame';
import { RecordMenu } from './RecordMenu';
import { RowChips } from './RowChips';
import { RowTitle } from './RowTitle';

const RecordRow = (props: RecordRowProps) => {
  const { head, title: given, onOpen, chips = [], age, trailing } = props;
  const untitled = given === '';
  const title = untitled ? head.code : given;
  const linked = onOpen !== undefined && rowLinked(title, chips);
  const code = <Text as="span" variant="caption" leading="16" tone="muted" mono>{head.code}</Text>;
  const opener = linked ? <Pressable onClick={onOpen} title={`Open ${head.code}`}>{code}</Pressable> : code;
  return (
    <RecordFrame
      name={head.code}
      kind={head.kind}
      density="row"
      code={untitled ? undefined : opener}
      title={<RowTitle title={title} board={head.boardKey} linked={linked} onOpen={onOpen} label={`Open ${head.code}: ${title}`} />}
      content={
        <>
          {chips.length > 0 && <RowChips chips={chips} board={head.boardKey} />}
          {age && <Text variant="caption" leading="16" tone="muted">{age}</Text>}
          {trailing}
          <AssigneeMark assignee={head.assignee} boardKey={head.boardKey} threads={head.threads} />
        </>
      }
      menu={<RecordMenu groups={head.menu} label={head.menuLabel} />}
      attrs={{ role: 'listitem', 'data-record': head.code }}
    />
  );
};

export { RecordRow };
