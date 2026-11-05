/* @layer renderer-app @kind component */
import { Box, TypeMark } from '@drizztdourden08/tessera';
import type { IconName } from '@drizztdourden08/tessera';
import { kindOf } from '../../../data/registry';
import { kindMark } from '../behavior/kind-color';
import type { TypeIconProps } from '../records.type';
import './FrameMark.css';

const TypeIcon = (props: TypeIconProps) => {
  const { kind: id, size = 'xs' } = props;
  const kind = kindOf(id);
  if (!kind) return null;
  return (
    <Box as="span" className="frame-mark" data-type-mark={kind.id}>
      <TypeMark icon={kind.icon as IconName} label={kind.label} {...kindMark(kind)} size={size} variant="outline" />
    </Box>
  );
};

export { TypeIcon };
