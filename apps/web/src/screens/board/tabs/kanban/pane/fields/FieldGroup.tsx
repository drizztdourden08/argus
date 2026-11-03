/* @layer renderer-app @kind component */
import { Flex } from '@drizztdourden08/tessera';
import { DrawerSection } from '../../sub-components/DrawerSection';
import type { FieldGroupProps } from '../pane.type';
import { FieldRow } from './FieldRow';

const FieldGroup = (props: FieldGroupProps) => {
  const { group, shared } = props;
  return (
    <DrawerSection title={group.label}>
      <Flex direction="column" gap="sm">
        {group.fields.map((name) => (
          <FieldRow key={name} name={name} {...shared} />
        ))}
      </Flex>
    </DrawerSection>
  );
};

export { FieldGroup };
