/* @layer renderer-app @kind component */
import { BrandMark } from '@drizztdourden08/tessera';
import type { OverlordMarkProps } from './OverlordMark.type';

const OverlordMark = (props: OverlordMarkProps) => {
  const { activity, size, className } = props;
  return <BrandMark app="argus" size={size} className={className} activity={activity === 'off' ? 'idle' : activity} />;
};

export { OverlordMark };
