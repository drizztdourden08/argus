/* @layer renderer-app @kind component */
import { Tag } from '@drizztdourden08/tessera';
import { Age } from '../../../../../shell';
import type { LauncherPartProps } from '../builds-props.type';

const RetiredChip = (props: LauncherPartProps) => {
  const { launcher } = props.view;
  if (!launcher.retiredAt) return null;
  return (
    <Tag title={launcher.retireNote ?? 'Retired: merged or replaced'}>
      retired <Age value={launcher.retiredAt} ago />{launcher.retiredBy ? ` by ${launcher.retiredBy}` : ''}
    </Tag>
  );
};

export { RetiredChip };
