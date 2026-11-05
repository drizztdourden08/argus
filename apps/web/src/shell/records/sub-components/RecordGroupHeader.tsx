/* @layer renderer-app @kind component */
import { Flex, Pressable, Tag, Text } from '@drizztdourden08/tessera';
import type { RecordGroupHeaderProps } from '../record-rows.type';
import { RecordLink } from './RecordLink';
import { TypeIcon } from './TypeIcon';

const RecordGroupHeader = (props: RecordGroupHeaderProps) => {
  const { kind, code, title, boardKey, onOpen, tag } = props;
  return (
    <Flex gap="sm" align="center">
      <TypeIcon kind={kind} />
      <RecordLink id={code} board={boardKey} />
      <Pressable onClick={onOpen} aria-label={`Open ${code}: ${title}`}>
        <Text weight={600} clamp={1} smartTimes>{title}</Text>
      </Pressable>
      {tag && <Tag title={tag.title}>{tag.text}</Tag>}
    </Flex>
  );
};

export { RecordGroupHeader };
