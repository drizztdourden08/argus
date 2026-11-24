/* @layer renderer-app @kind component */
import { BoardIcon } from '../../../../shell/BoardIcon';
import { pickerValue } from '../behavior/picker-value';
import { useIconChoice } from '../behavior/useIconChoice';
import { ICON_NOTE, ICON_PICKER_LABEL, ICON_TYPES } from '../BoardSettings.constants';
import type { SectionProps } from '../BoardSettings.type';
import { LookPicker } from './LookPicker';

const IconSection = (props: SectionProps) => {
  const { boardKey } = props;
  const choice = useIconChoice(boardKey);
  return (
    <LookPicker
      title="Board icon"
      note={ICON_NOTE}
      preview={<BoardIcon boardKey={boardKey} icon={choice.setting} avatar={choice.avatar} size="lg" title="Board icon preview" />}
      pickerLabel={ICON_PICKER_LABEL}
      value={pickerValue(choice.setting)}
      accept={ICON_TYPES}
      uploadLabel="Upload a PNG or SVG"
      choice={choice}
    />
  );
};

export { IconSection };
