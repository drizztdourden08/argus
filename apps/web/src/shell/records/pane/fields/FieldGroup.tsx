/* @layer renderer-app @kind component */
import { Flex } from '@drizztdourden08/tessera';
import { Fragment } from 'react';
import { DrawerSection } from '../../../../screens/board/tabs/kanban/sub-components/DrawerSection';
import type { FieldGroupProps } from './fields.type';

const FieldGroup = (props: FieldGroupProps) => {
  const { group, render } = props;
  return (
    <DrawerSection title={group.label}>
      <Flex direction="column" gap="sm">
        {group.fields.map((name) => <Fragment key={name}>{render(name)}</Fragment>)}
      </Flex>
    </DrawerSection>
  );
};

export { FieldGroup };
