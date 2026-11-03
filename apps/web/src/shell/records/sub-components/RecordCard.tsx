/* @layer renderer-app @kind component */
import { Flex, Pressable, Text } from '@drizztdourden08/tessera';
import { hasRecords } from '../behavior/has-records';
import type { RecordCardProps } from '../records.type';
import { LinkedTitle } from './LinkedTitle';
import { RecordHead } from './RecordHead';

const RecordCard = (props: RecordCardProps) => {
  const { head, title, openLabel, onOpen, signals, foot, attrs, onClickCapture } = props;
  return (
    <Flex direction="column" gap="xs" {...attrs} onClickCapture={onClickCapture}>
      <RecordHead {...head} />
      {hasRecords(title) ? (
        <LinkedTitle text={title} board={head.boardKey} onOpen={onOpen} weight={500} clamp={3} label={openLabel} />
      ) : (
        <Pressable onClick={onOpen} aria-label={openLabel}>
          <Text weight={500} clamp={3} smartTimes>{title}</Text>
        </Pressable>
      )}
      {signals}
      {foot}
    </Flex>
  );
};

export { RecordCard };
