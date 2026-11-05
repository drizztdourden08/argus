/* @layer renderer-app @kind component */
import { Flex, Pressable, Tag, Text } from '@drizztdourden08/tessera';
import type { RecordGroupHeaderProps } from '../record-rows.type';
import { FrameSlot } from './FrameSlot';
import { RecordLink } from './RecordLink';
import { TypeIcon } from './TypeIcon';

const RecordGroupHeader = (props: RecordGroupHeaderProps) => {
  const { kind, code, title, boardKey, onOpen, tag } = props;
  return (
    <Flex gap="sm" align="center">
      <FrameSlot><TypeIcon kind={kind} /></FrameSlot>
      <FrameSlot><RecordLink id={code} board={boardKey} /></FrameSlot>
      <FrameSlot>
        <Pressable onClick={onOpen} aria-label={`Open ${code}: ${title}`}>
          <Text weight={600} clamp={1} smartTimes>{title}</Text>
        </Pressable>
      </FrameSlot>
      {tag && <Tag title={tag.title}>{tag.text}</Tag>}
    </Flex>
  );
};

export { RecordGroupHeader };
