/* @layer renderer-app @kind component */
import { SettingsSection } from '@drizztdourden08/tessera';
import { useProfileChoice } from '../behavior/useProfileChoice';
import { AVATAR_NOTE, INITIALS_LENGTH, INITIALS_NOTE, NAME_NOTE } from '../AppSettings.constants';
import { AvatarRow } from './AvatarRow';
import { ProfileText } from './ProfileText';

const ProfileSection = () => {
  const choice = useProfileChoice();
  return (
    <SettingsSection
      rows={[
        { id: 'avatar', title: 'Avatar', description: AVATAR_NOTE, content: <AvatarRow choice={choice} /> },
        {
          id: 'name',
          title: 'Display name',
          description: NAME_NOTE,
          input: {
            kind: 'custom',
            control: <ProfileText key={choice.name} label="Display name" value={choice.name} disabled={choice.busy} onSave={(next) => void choice.saveName(next)} />,
          },
        },
        {
          id: 'initials',
          title: 'Initials',
          description: INITIALS_NOTE,
          input: {
            kind: 'custom',
            control: (
              <ProfileText
                key={choice.initials}
                label="Initials"
                value={choice.initials}
                maxLength={INITIALS_LENGTH}
                disabled={choice.busy}
                onSave={(next) => void choice.saveInitials(next)}
              />
            ),
          },
        },
      ]}
    />
  );
};

export { ProfileSection };
