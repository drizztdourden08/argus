/* @layer renderer-app @kind component */
import { DateText, Tag } from '@drizztdourden08/tessera';
import type { LauncherPartProps } from '../builds-props.type';

const RetiredChip = (props: LauncherPartProps) => {
  const { launcher } = props.view;
  if (!launcher.retiredAt) return null;
  return (
    <Tag title={launcher.retireNote ?? 'Retired: merged or replaced'}>
      retired <DateText value={launcher.retiredAt} format="relative" />{launcher.retiredBy ? ` by ${launcher.retiredBy}` : ''}
    </Tag>
  );
};

export { RetiredChip };
