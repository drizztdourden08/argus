/* @layer renderer-app @kind component */
import { Tag } from '@drizztdourden08/tessera';
import type { DepTagProps } from './DepTag.type';

const DepTag = (props: DepTagProps) => {
  const { color, title, children } = props;
  if (color === 'neutral') return <Tag title={title}>{children}</Tag>;
  return <Tag variant="urgency" color={color} title={title}>{children}</Tag>;
};

export { DepTag };
