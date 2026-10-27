/* @layer renderer-app @kind component */
import { Drawer, Flex } from '@drizztdourden08/tessera';
import type { RecordDrawerProps } from '../shared.type';
import { RecordHeader } from './RecordHeader';

const RecordDrawer = (props: RecordDrawerProps) => {
  const { open, label, wide, eyebrow, title, tools, onClose, children } = props;
  return (
    <Drawer open={open} onClose={onClose} size={wide ? 'xl' : 'lg'} label={label}>
      <Flex direction="column" gap="lg">
        <RecordHeader eyebrow={eyebrow} title={title} tools={tools} onClose={onClose} />
        {children}
      </Flex>
    </Drawer>
  );
};

export { RecordDrawer };
