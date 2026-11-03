/* @layer renderer-app @kind component */
import { Flex, Text } from '@drizztdourden08/tessera';
import { fieldOf } from '../../../../data/registry';
import { fieldLabel } from '../model/field-label';
import type { FieldFrameProps } from './fields.type';

const FieldFrame = (props: FieldFrameProps) => {
  const { name, children } = props;
  const spec = fieldOf(name);
  if (!spec) return null;
  return (
    <Flex direction="column" gap="xs">
      <Text variant="caption" tone="muted">{`${fieldLabel(name)}${spec.required ? ' *' : ''}`}</Text>
      {children}
    </Flex>
  );
};

export { FieldFrame };
