/* @layer renderer-app @kind component */
import { Flex, List, ListItem, Status, Text } from '@drizztdourden08/tessera';
import { itemCode } from '../../../../../shell';
import type { AcListProps } from '../tests-drawer-props.type';
import { DrawerSection } from './DrawerSection';

const AcList = (props: AcListProps) => {
  const { test, item } = props;
  if (test.ac.length === 0 || !item) return null;
  return (
    <DrawerSection title={`Covers on ${itemCode(item)}`}>
      <List aria-label="Acceptance criteria">
        {test.ac.map((number) => {
          const criterion = item.ac?.[number - 1];
          return (
            <ListItem key={number}>
              <Flex gap="sm" align="center" wrap>
                <Status tone={criterion?.done ? 'success' : 'neutral'} dot>{criterion?.done ? 'done' : 'open'}</Status>
                <Text>{`AC${number}: ${criterion?.text ?? ''}`}</Text>
              </Flex>
            </ListItem>
          );
        })}
      </List>
    </DrawerSection>
  );
};

export { AcList };
