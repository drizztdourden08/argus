/* @layer renderer-app @kind component */
import { Pressable, RecordHeader, Tag, Text } from '@drizztdourden08/tessera';
import type { RecordGroupHeaderProps } from '../record-rows.type';
import { RecordLink } from './RecordLink';
import { TypeIcon } from './TypeIcon';

const RecordGroupHeader = (props: RecordGroupHeaderProps) => {
  const { kind, code, title, boardKey, onOpen, tag } = props;
  return (
    <RecordHeader
      density="row"
      icon={<TypeIcon kind={kind} />}
      code={<RecordLink id={code} board={boardKey} />}
      title={(
        <Pressable onClick={onOpen} aria-label={`Open ${code}: ${title}`}>
          <Text weight={600} clamp={1} smartTimes>{title}</Text>
        </Pressable>
      )}
      content={tag && <Tag title={tag.title}>{tag.text}</Tag>}
    />
  );
};

export { RecordGroupHeader };
