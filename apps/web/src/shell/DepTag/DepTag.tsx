/* @layer renderer-app @kind component */
import { Tag } from '@drizztdourden08/tessera';
import type { DepTagProps } from './DepTag.type';

const DepTag = (props: DepTagProps) => {
  const { color, icon, title, children } = props;
  if (color === 'neutral') return <Tag icon={icon} title={title}>{children}</Tag>;
  return <Tag variant="urgency" color={color} icon={icon} title={title}>{children}</Tag>;
};

export { DepTag };
