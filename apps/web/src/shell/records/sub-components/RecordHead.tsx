/* @layer renderer-app @kind component */
import type { RecordHeadViewProps } from '../records.type';
import { AssigneeMark } from './AssigneeMark';
import { RecordFrame } from './RecordFrame';
import { RecordLink } from './RecordLink';
import { RecordMenu } from './RecordMenu';
import { RecordTime } from './RecordTime';
import { TypeIcon } from './TypeIcon';

const RecordHead = (props: RecordHeadViewProps) => {
  const { kind, code, assignee, boardKey, threads, menu, menuLabel, mark, size, selection, content, time, ...frame } = props;
  return (
    <RecordFrame
      {...frame}
      name={code}
      icon={mark ?? (size ? <TypeIcon kind={kind} size={size} /> : undefined)}
      kind={kind}
      selection={selection}
      code={<RecordLink id={code} />}
      content={
        <>
          {content}
          <RecordTime time={time} />
          <AssigneeMark assignee={assignee} boardKey={boardKey} threads={threads} size={size} />
        </>
      }
      menu={<RecordMenu groups={menu} label={menuLabel} />}
    />
  );
};

export { RecordHead };
