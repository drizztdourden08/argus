/* @layer renderer-app @kind component */
import { Flex, Text } from '@drizztdourden08/tessera';
import type { GroupTitleProps } from '../builds-props.type';

const GroupTitle = (props: GroupTitleProps) => {
  const { title, note } = props;
  return (
    <Flex gap="sm" align="center" wrap>
      <Text weight={600}>{title}</Text>
      <Text variant="caption" tone="dim">{note}</Text>
    </Flex>
  );
};

export { GroupTitle };
