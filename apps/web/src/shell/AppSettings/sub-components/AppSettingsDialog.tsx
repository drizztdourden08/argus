/* @layer renderer-app @kind component */
import { Button, DialogShell, SegmentedControl } from '@drizztdourden08/tessera';
import { SECTIONS } from '../AppSettings.constants';
import type { AppSettingsDialogProps } from '../AppSettings.type';
import { sectionView } from '../behavior/section-view';

const AppSettingsDialog = (props: AppSettingsDialogProps) => {
  const { section, onSection, onClose } = props;
  const View = sectionView(section);
  return (
    <DialogShell
      open
      steady
      size="lg"
      surface="glass"
      title="Settings"
      onClose={onClose}
      headerExtra={<SegmentedControl size="sm" aria-label="Settings section" value={section} options={SECTIONS} onChange={onSection} />}
      actions={<Button size="sm" variant="ghost" onClick={onClose}>Close</Button>}
    >
      <View />
    </DialogShell>
  );
};

export { AppSettingsDialog };
