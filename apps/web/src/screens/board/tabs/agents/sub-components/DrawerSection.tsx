/* @layer renderer-app @kind component */
import { Flex, Text } from '@drizztdourden08/tessera';
import type { DrawerSectionProps } from '../AgentsTab.type';

const DrawerSection = (props: DrawerSectionProps) => {
  const { title, children } = props;
  return (
    <Flex direction="column" gap="xs">
      <Text variant="overline" tone="muted">{title}</Text>
      {children}
    </Flex>
  );
};

export { DrawerSection };
