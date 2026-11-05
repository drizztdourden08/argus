/* @layer renderer-app @kind component */
import { Card, Divider, Flex, Text } from '@drizztdourden08/tessera';
import type { PanelProps } from './shell.type';

const Panel = (props: PanelProps) => {
  const { title, note, tools, children } = props;
  return (
    <Card surface="glass">
      <Flex direction="column" gap="sm">
        <Flex justify="between" align="center" gap="md" wrap>
          <Flex gap="sm" align="center" wrap>
            <Text as="h2" variant="title">{title}</Text>
            {note != null && <Text variant="caption" tone="muted">{note}</Text>}
          </Flex>
          {tools}
        </Flex>
        <Divider />
        {children}
      </Flex>
    </Card>
  );
};

export { Panel };
