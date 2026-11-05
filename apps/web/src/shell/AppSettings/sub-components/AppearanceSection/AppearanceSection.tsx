/* @layer renderer-app @kind component */
import { SettingsSection } from '@drizztdourden08/tessera';
import { FINISHED_KEY, HIDDEN_KEY, WIDE_KEY, useStoredFlag } from '../../../../stored';

const AppearanceSection = () => {
  const [wide, setWide] = useStoredFlag(WIDE_KEY);
  const [hidden, setHidden] = useStoredFlag(HIDDEN_KEY);
  const [finished, setFinished] = useStoredFlag(FINISHED_KEY);
  return (
    <SettingsSection
      id="appearance"
      rows={[
        { id: 'wide', title: 'Wide record drawer', noDescription: true, input: { kind: 'toggle', value: wide, onChange: setWide } },
        {
          id: 'hidden',
          title: 'Show hidden threads',
          description: 'Done, archived and unnamed sessions.',
          input: { kind: 'toggle', value: hidden, onChange: setHidden },
        },
        { id: 'finished', title: 'Show finished agents', noDescription: true, input: { kind: 'toggle', value: finished, onChange: setFinished } },
      ]}
    />
  );
};

export { AppearanceSection };
