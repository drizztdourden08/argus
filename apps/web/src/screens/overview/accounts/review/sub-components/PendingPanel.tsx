/* @layer renderer-app @kind component */
import { Inline, Tag, Text } from '@drizztdourden08/tessera';
import { editLabel } from '../model';
import type { PendingPanelProps } from '../review-props.type';

const PendingPanel = (props: PendingPanelProps) => {
  const { items, names, changed, onRemove } = props;
  if (items.length === 0 && changed === 0) return null;
  return (
    <Inline gap="sm" align="center" wrap>
      <Text variant="overline">Pending changes</Text>
      {items.map((item) => <Tag key={item.key} onRemove={() => onRemove(item.key)}>{editLabel(item.edit, names)}</Tag>)}
      {changed > 0 && <Text variant="caption">{`${changed} choices changed from default`}</Text>}
    </Inline>
  );
};

export { PendingPanel };
