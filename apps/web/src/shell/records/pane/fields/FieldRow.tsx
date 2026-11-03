/* @layer renderer-app @kind component */
import { Flex, Text } from '@drizztdourden08/tessera';
import { fieldOf } from '../../../../data/registry';
import { fieldLabel } from '../model/field-label';
import type { FieldProps } from '../pane.type';
import { FieldEdit } from './FieldEdit';

const FieldRow = (props: FieldProps) => {
  const { name } = props;
  const spec = fieldOf(name);
  if (!spec) return null;
  return (
    <Flex direction="column" gap="xs">
      <Text variant="caption" tone="muted">{`${fieldLabel(name)}${spec.required ? ' *' : ''}`}</Text>
      <FieldEdit {...props} />
    </Flex>
  );
};

export { FieldRow };
