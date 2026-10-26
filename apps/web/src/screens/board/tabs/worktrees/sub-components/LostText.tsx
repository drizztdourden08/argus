/* @layer renderer-app @kind component */
import { Flex, List, ListItem, Text } from '@drizztdourden08/tessera';
import { lostItems } from '../model';
import type { LostTextProps } from '../worktrees-props.type';

const LostText = (props: LostTextProps) => {
  const { row } = props;
  return (
    <Flex direction="column" gap="xs">
      <Text><Text.B>Delete <Text.Code>{row.path}</Text.Code> now?</Text.B> This discards:</Text>
      <List>
        {lostItems(row).map((item) => <ListItem key={item}>{item}</ListItem>)}
      </List>
      <Text>A backup bundle is saved first (with a patch of the uncommitted changes); the answer says where it is and how to restore it.</Text>
    </Flex>
  );
};

export { LostText };
