/* @layer renderer-app @kind component */
import { Divider, Flex, Icon, IconButton, Title } from '@drizztdourden08/tessera';
import type { RecordHeaderProps } from '../shared.type';

const RecordHeader = (props: RecordHeaderProps) => {
  const { eyebrow, title, tools, onClose } = props;
  return (
    <Flex direction="column" gap="xs">
      <Flex justify="between" align="center" gap="sm" wrap>
        <Flex gap="xs" align="center" wrap>{eyebrow}</Flex>
        <Flex gap="xs" align="center" role="toolbar" aria-label="Drawer tools">
          {tools}
          <IconButton label="Close" title="Close (Esc)" size="xs" variant="ghost" onClick={onClose}>
            <Icon name="x" />
          </IconButton>
        </Flex>
      </Flex>
      <Title level={4}>{title}</Title>
      <Divider />
    </Flex>
  );
};

export { RecordHeader };
