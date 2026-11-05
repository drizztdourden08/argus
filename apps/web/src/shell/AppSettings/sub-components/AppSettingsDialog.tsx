/* @layer renderer-app @kind component */
import { Button, DialogShell, SegmentedControl } from '@drizztdourden08/tessera';
import { SECTIONS } from '../AppSettings.constants';
import type { AppSettingsDialogProps } from '../AppSettings.type';
import { SECTION_VIEWS } from '../section-views';

const AppSettingsDialog = (props: AppSettingsDialogProps) => {
  const { section, onSection, onClose } = props;
  const View = SECTION_VIEWS[section];
  return (
    <DialogShell
      open
      steady
      size="lg"
      surface="glass"
      title="Settings"
      onClose={onClose}
      headerExtra={<SegmentedControl selectedTone="neutral" size="sm" aria-label="Settings section" value={section} options={SECTIONS} onChange={onSection} />}
      actions={<Button size="sm" variant="ghost" onClick={onClose}>Close</Button>}
    >
      <View />
    </DialogShell>
  );
};

export { AppSettingsDialog };
