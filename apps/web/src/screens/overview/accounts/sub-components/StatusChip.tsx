/* @layer renderer-app @kind component */
import { Tag } from '@drizztdourden08/tessera';
import type { StatusChipProps } from '../compare.type';

const StatusChip = (props: StatusChipProps) => {
  const { look, title } = props;
  if (look.tone === 'neutral') return <Tag title={title}>{look.label}</Tag>;
  return <Tag variant="urgency" color={look.tone} fill="tint" title={title}>{look.label}</Tag>;
};

export { StatusChip };
