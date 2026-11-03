/* @layer renderer-app @kind component */
import { Pressable, Text } from '@drizztdourden08/tessera';
import { parseRecord } from '../behavior/parse-record';
import type { RowCodeProps } from '../record-rows.type';
import { RecordLink } from './RecordLink';

const RowCode = (props: RowCodeProps) => {
  const { code, board, linked, onOpen } = props;
  const text = <Text as="span" variant="caption" tone="muted" mono>{code}</Text>;
  if (!onOpen) return text;
  if (linked) return <Pressable onClick={onOpen} title={`Open ${code}`}>{text}</Pressable>;
  return parseRecord(code) ? <RecordLink id={code} board={board}>{text}</RecordLink> : text;
};

export { RowCode };
