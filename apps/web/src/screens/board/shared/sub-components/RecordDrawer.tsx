/* @layer renderer-app @kind component */
import { Drawer, Flex } from '@drizztdourden08/tessera';
import type { RecordDrawerProps } from '../shared.type';

const RecordDrawer = (props: RecordDrawerProps) => {
  const { open, label, wide, eyebrow, title, tools, onClose, children } = props;
  return (
    <Drawer open={open} onClose={onClose} look="record" stackNarrow size={wide ? 'xl' : 'wide'} label={label} eyebrow={eyebrow} title={title} headerActions={tools}>
      <Flex direction="column" gap="lg">{children}</Flex>
    </Drawer>
  );
};

export { RecordDrawer };
