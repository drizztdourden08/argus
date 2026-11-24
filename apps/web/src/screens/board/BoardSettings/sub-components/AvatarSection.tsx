/* @layer renderer-app @kind component */
import { AVATAR_TYPES, resolveAvatar } from '../../../../avatar';
import { BoardIcon } from '../../../../shell/BoardIcon';
import { pickerValue } from '../behavior/picker-value';
import { useAvatarChoice } from '../behavior/useAvatarChoice';
import { AVATAR_NOTE, AVATAR_PICKER_LABEL } from '../BoardSettings.constants';
import type { SectionProps } from '../BoardSettings.type';
import { LookPicker } from './LookPicker';

const AvatarSection = (props: SectionProps) => {
  const choice = useAvatarChoice(props.boardKey);
  const source = resolveAvatar(props.boardKey, choice.setting);
  return (
    <LookPicker
      title="Lead avatar"
      note={AVATAR_NOTE}
      preview={source && <BoardIcon source={source} size="lg" title="Lead avatar preview" />}
      pickerLabel={AVATAR_PICKER_LABEL}
      value={pickerValue(choice.setting)}
      accept={AVATAR_TYPES}
      uploadLabel="Upload an image"
      choice={choice}
    />
  );
};

export { AvatarSection };
