/* @layer renderer-app @kind component */
import { Text } from '@drizztdourden08/tessera';
import { Age, RecordLink } from '../../../../../shell';
import type { RetiredNoteProps } from '../builds-props.type';

const RetiredNote = (props: RetiredNoteProps) => {
  const { view, boardKey } = props;
  const { launcher } = view;
  if (!launcher.retiredAt) return null;
  const item = launcher.retiredByItem ?? launcher.for;
  return (
    <Text variant="caption" tone="muted" title={launcher.retireNote ?? 'Retired: merged or replaced'}>
      {item && <>{launcher.retiredByItem ? 'closed with ' : 'was for '}<RecordLink id={item} board={boardKey} />{' · '}</>}
      {'retired '}<Age value={launcher.retiredAt} ago />{launcher.retiredBy ? ` by ${launcher.retiredBy}` : ''}
    </Text>
  );
};

export { RetiredNote };
