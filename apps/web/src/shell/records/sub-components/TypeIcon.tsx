/* @layer renderer-app @kind component */
import { Box, TypeMark } from '@drizztdourden08/tessera';
import type { IconName } from '@drizztdourden08/tessera';
import { kindOf } from '../../../data/registry';
import { kindColor } from '../behavior/kind-color';
import type { TypeIconProps } from '../records.type';

const TypeIcon = (props: TypeIconProps) => {
  const { kind: id, size = 'xs' } = props;
  const kind = kindOf(id);
  if (!kind) return null;
  return (
    <Box as="span" data-type-mark={kind.id}>
      <TypeMark icon={kind.icon as IconName} label={kind.label} color={kindColor(kind)} size={size} variant="outline" />
    </Box>
  );
};

export { TypeIcon };
