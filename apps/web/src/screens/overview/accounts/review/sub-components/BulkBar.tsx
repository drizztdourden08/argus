/* @layer renderer-app @kind component */
import { Button, Inline, Text } from '@drizztdourden08/tessera';
import type { BulkBarProps } from '../review-props.type';

const BulkBar = (props: BulkBarProps) => {
  const { label, total, archive, unarchive, remove, onArchive, onUnarchive, onMove, onDelete, onClear } = props;
  return (
    <Inline gap="sm" align="center" wrap>
      <Text variant="overline">{`${total} selected on ${label}`}</Text>
      <Button size="sm" variant="ghost" disabled={archive === 0} onClick={onArchive}>Archive</Button>
      <Button size="sm" variant="ghost" disabled={unarchive === 0} onClick={onUnarchive}>Unarchive</Button>
      <Button size="sm" variant="ghost" onClick={onMove}>Move to folder</Button>
      <Button size="sm" variant="ghost" disabled={remove === 0} title="Only archived chats can be deleted" onClick={onDelete}>Delete</Button>
      <Button size="sm" variant="ghost" onClick={onClear}>Clear selection</Button>
    </Inline>
  );
};

export { BulkBar };
