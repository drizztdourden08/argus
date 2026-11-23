/* @layer renderer-app @kind component */
import { Button, DialogShell, Divider, Flex } from '@drizztdourden08/tessera';
import { ColumnsRulesSection } from '../../tabs/kanban';
import type { BoardSettingsDialogProps } from '../BoardSettings.type';
import { AvatarSection } from './AvatarSection';
import { IconSection } from './IconSection';
import { TitleLimitSection } from './TitleLimitSection';

const BoardSettingsDialog = (props: BoardSettingsDialogProps) => {
  const { boardKey, onClose } = props;
  return (
    <DialogShell
      open
      size="md"
      surface="glass"
      title="Board settings"
      onClose={onClose}
      actions={<Button size="sm" variant="ghost" onClick={onClose}>Close</Button>}
    >
      <Flex direction="column" gap="lg">
        <IconSection boardKey={boardKey} />
        <Divider />
        <AvatarSection boardKey={boardKey} />
        <Divider />
        <TitleLimitSection boardKey={boardKey} />
        <Divider />
        <ColumnsRulesSection boardKey={boardKey} />
      </Flex>
    </DialogShell>
  );
};

export { BoardSettingsDialog };
