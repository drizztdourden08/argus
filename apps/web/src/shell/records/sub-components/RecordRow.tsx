/* @layer renderer-app @kind component */
import { Flex, Pressable } from '@drizztdourden08/tessera';
import { rowLinked } from '../behavior/row-linked';
import type { RecordRowProps } from '../record-rows.type';
import { AssigneeMark } from './AssigneeMark';
import { FrameCode } from './FrameCode';
import { RecordFrame } from './RecordFrame';
import { MenuSlot } from './MenuSlot';
import { RecordTime } from './RecordTime';
import { RowChips } from './RowChips';
import { RowTitle } from './RowTitle';

const RecordRow = (props: RecordRowProps) => {
  const { head, title: given, onOpen, chips = [], time, compact, trailing, body, open, onOpenChange } = props;
  const untitled = given === '';
  const title = untitled ? head.code : given;
  const linked = onOpen !== undefined && rowLinked(title, chips);
  const code = <FrameCode>{head.code}</FrameCode>;
  const opener = linked ? <Pressable onClick={onOpen} title={`Open ${head.code}`}>{code}</Pressable> : code;
  return (
    <RecordFrame
      name={head.code}
      kind={head.kind}
      density={compact ? 'compact' : 'row'}
      open={open}
      onOpenChange={onOpenChange}
      code={untitled ? undefined : opener}
      title={<RowTitle title={title} board={head.boardKey} linked={linked} onOpen={onOpen} label={`Open ${head.code}: ${title}`} />}
      content={
        <Flex gap="xs" align="center">
          {chips.length > 0 && <RowChips chips={chips} board={head.boardKey} />}
          {trailing}
          <RecordTime time={time} />
          <AssigneeMark assignee={head.assignee} boardKey={head.boardKey} threads={head.threads} />
        </Flex>
      }
      menu={<MenuSlot groups={head.menu} label={head.menuLabel} />}
      attrs={{ role: 'listitem', 'data-record': head.code }}
    >
      {body}
    </RecordFrame>
  );
};

export { RecordRow };
