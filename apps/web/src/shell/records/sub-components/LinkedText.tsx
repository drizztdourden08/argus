/* @layer renderer-app @kind component */
import { splitRecords } from '../behavior/split-records';
import type { LinkedTextProps } from '../records.type';
import { RecordLink } from './RecordLink';
import { ShownCode } from './ShownCode';

const LinkedText = (props: LinkedTextProps) => {
  const { text, board } = props;
  return (
    <>
      {splitRecords(text).map((part, index) =>
        part.ref ? (
          <RecordLink key={`${index}:${part.text}`} id={part.text} board={board}><ShownCode id={part.text} board={board} /></RecordLink>
        ) : (
          part.text
        ),
      )}
    </>
  );
};

export { LinkedText };
