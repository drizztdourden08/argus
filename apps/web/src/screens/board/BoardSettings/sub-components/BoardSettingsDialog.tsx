/* @layer renderer-app @kind component */
import { Button, DialogShell, Flex, Tabs } from '@drizztdourden08/tessera';
import { useState } from 'react';
import { ColumnsRulesSection } from '../../tabs/kanban';
import { SETTINGS_TABS } from '../BoardSettings.constants';
import type { BoardSettingsDialogProps, SettingsTab } from '../BoardSettings.type';
import { AppearanceTab } from './AppearanceTab';
import { TitleLimitSection } from './TitleLimitSection';

const BoardSettingsDialog = (props: BoardSettingsDialogProps) => {
  const { boardKey, onClose } = props;
  const [tab, setTab] = useState<SettingsTab>('appearance');
  return (
    <DialogShell
      open
      size="lg"
      surface="glass"
      title="Board settings"
      onClose={onClose}
      actions={<Button size="sm" variant="ghost" onClick={onClose}>Close</Button>}
    >
      <Flex direction="column" gap="lg">
        <Tabs tabs={SETTINGS_TABS} activeTab={tab} selectedTone="neutral" onTabChange={(id) => setTab(id as SettingsTab)} />
        {tab === 'appearance' && <AppearanceTab boardKey={boardKey} />}
        {tab === 'general' && <TitleLimitSection boardKey={boardKey} />}
        {tab === 'kanban' && <ColumnsRulesSection boardKey={boardKey} />}
      </Flex>
    </DialogShell>
  );
};

export { BoardSettingsDialog };
