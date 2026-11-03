/* @layer renderer-app @kind component */
import { Pressable, Text } from '@drizztdourden08/tessera';
import { LinkedTitle } from './LinkedTitle';
import type { RowTitleProps } from '../record-rows.type';

const RowTitle = (props: RowTitleProps) => {
  const { title, board, linked, onOpen, label } = props;
  if (!onOpen) return <Text clamp={1} smartTimes>{title}</Text>;
  if (linked) return <LinkedTitle text={title} board={board} onOpen={onOpen} clamp={1} label={label} />;
  return (
    <Pressable onClick={onOpen} aria-label={label}>
      <Text clamp={1} smartTimes>{title}</Text>
    </Pressable>
  );
};

export { RowTitle };
