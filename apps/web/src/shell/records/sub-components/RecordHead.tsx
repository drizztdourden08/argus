/* @layer renderer-app @kind component */
import { Checkbox, Flex, Spacer } from '@drizztdourden08/tessera';
import type { RecordHeadProps } from '../records.type';
import { AssigneeMark } from './AssigneeMark';
import { RecordLink } from './RecordLink';
import { RecordMenu } from './RecordMenu';
import { TypeIcon } from './TypeIcon';

const RecordHead = (props: RecordHeadProps) => {
  const { kind, code, assignee, boardKey, threads, menu, menuLabel, title, size = 'xs', selection } = props;
  return (
    <Flex gap="xs" align="center">
      {selection?.selecting ? (
        <Flex onClickCapture={(event) => selection.onMark(event.shiftKey)}>
          <Checkbox checked={selection.selected} onChange={selection.onToggle} ariaLabel={`Select ${code}`} size="sm" />
        </Flex>
      ) : (
        <TypeIcon kind={kind} size={size} />
      )}
      <RecordLink id={code} />
      {title}
      <Spacer />
      <AssigneeMark assignee={assignee} boardKey={boardKey} threads={threads} size={size} />
      <RecordMenu groups={menu} label={menuLabel} />
    </Flex>
  );
};

export { RecordHead };
