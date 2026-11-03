/* @layer renderer-app @kind component */
import { Box, Flex, Pressable, Tag, Text } from '@drizztdourden08/tessera';
import type { MouseEvent } from 'react';
import type { RecordGroupHeaderProps } from '../record-rows.type';
import { RecordLink } from './RecordLink';
import { TypeIcon } from './TypeIcon';

const keepOpen = (event: MouseEvent): void => event.preventDefault();

const RecordGroupHeader = (props: RecordGroupHeaderProps) => {
  const { kind, code, title, boardKey, onOpen, tag } = props;
  return (
    <Flex gap="sm" align="center">
      <TypeIcon kind={kind} />
      <Box as="span" onClick={keepOpen}>
        <RecordLink id={code} board={boardKey} />
      </Box>
      <Box as="span" onClick={keepOpen}>
        <Pressable onClick={onOpen} aria-label={`Open ${code}: ${title}`}>
          <Text weight={600} clamp={1} smartTimes>{title}</Text>
        </Pressable>
      </Box>
      {tag && <Tag title={tag.title}>{tag.text}</Tag>}
    </Flex>
  );
};

export { RecordGroupHeader };
