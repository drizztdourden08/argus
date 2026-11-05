/* @layer renderer-app @kind component */
import { Pressable, Text } from '@drizztdourden08/tessera';
import { hasRecords } from '../behavior/has-records';
import type { CardTitleProps } from '../record-rows.type';
import { LinkedTitle } from './LinkedTitle';

const CardTitle = (props: CardTitleProps) => {
  const { title, board, label, onOpen, clamp } = props;
  if (hasRecords(title)) return <LinkedTitle text={title} board={board} onOpen={onOpen} weight={500} clamp={clamp} label={label} />;
  return (
    <Pressable onClick={onOpen} aria-label={label}>
      <Text weight={500} clamp={clamp} smartTimes>{title}</Text>
    </Pressable>
  );
};

export { CardTitle };
