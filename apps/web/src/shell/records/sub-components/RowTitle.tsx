/* @layer renderer-app @kind component */
import { Pressable, Text } from '@drizztdourden08/tessera';
import { LinkedTitle } from './LinkedTitle';
import type { RowTitleProps } from '../record-rows.type';
import './RowTitle.css';

const RowTitle = (props: RowTitleProps) => {
  const { title, board, linked, onOpen, label } = props;
  if (!onOpen) return <Text smartTimes title={title}>{title}</Text>;
  if (linked) return <LinkedTitle text={title} board={board} onOpen={onOpen} className="row-title" label={label} />;
  return (
    <Pressable onClick={onOpen} aria-label={label} className="row-title">
      <Text smartTimes>{title}</Text>
    </Pressable>
  );
};

export { RowTitle };
