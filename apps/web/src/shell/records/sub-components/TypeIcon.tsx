/* @layer renderer-app @kind component */
import { Flex, TypeMark } from '@drizztdourden08/tessera';
import type { IconName } from '@drizztdourden08/tessera';
import { kindOf } from '../../../data/registry';
import { kindMark } from '../behavior/kind-color';
import type { TypeIconProps } from '../records.type';

const TypeIcon = (props: TypeIconProps) => {
  const { kind: id, size = 'xs' } = props;
  const kind = kindOf(id);
  if (!kind) return null;
  return (
    <Flex as="span" inline align="center" data-type-mark={kind.id}>
      <TypeMark icon={kind.icon as IconName} label={kind.label} {...kindMark(kind)} size={size} variant="outline" />
    </Flex>
  );
};

export { TypeIcon };
