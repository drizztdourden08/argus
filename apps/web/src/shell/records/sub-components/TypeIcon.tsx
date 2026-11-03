/* @layer renderer-app @kind component */
import { Box, Icon, Tooltip } from '@drizztdourden08/tessera';
import type { IconName } from '@drizztdourden08/tessera';
import { kindOf } from '../../../data/registry';
import { categoryColor } from '../behavior/category-color';
import { kindColor } from '../behavior/kind-color';
import { HEAD_SIZES } from '../head.constants';
import type { TypeIconProps } from '../records.type';

const TypeIcon = (props: TypeIconProps) => {
  const { kind: id, size = 'xs' } = props;
  const kind = kindOf(id);
  if (!kind) return null;
  const hue = categoryColor(kind.look.color);
  return (
    <Tooltip content={kind.label}>
      <Box as="span" data-type-mark={kind.id}>
        <Icon name={kind.icon as IconName} size={HEAD_SIZES[size].px} hue={hue} color={hue ? undefined : kindColor(kind)} label={kind.label} />
      </Box>
    </Tooltip>
  );
};

export { TypeIcon };
