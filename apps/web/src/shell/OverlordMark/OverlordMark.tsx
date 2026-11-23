/* @layer renderer-app @kind component */
import { BrandMark } from '@drizztdourden08/tessera';
import type { OverlordMarkProps } from './OverlordMark.type';
import './OverlordMark.css';

const OverlordMark = (props: OverlordMarkProps) => {
  const { activity, size, className } = props;
  const own = `overlord-mark is-${activity}`;
  return <BrandMark app="argus" size={size} className={className ? `${className} ${own}` : own} />;
};

export { OverlordMark };
