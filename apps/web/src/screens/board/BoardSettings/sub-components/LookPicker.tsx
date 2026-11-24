/* @layer renderer-app @kind component */
import { DropZone, Flex, IconPicker, Text } from '@drizztdourden08/tessera';
import { BOARD_ICON_NAMES, toBoardIconName } from '../../../../board-icon';
import { AvatarReset } from '../../../../shell/AvatarReset';
import type { LookPickerProps } from '../BoardSettings.type';

const LookPicker = (props: LookPickerProps) => {
  const { title, note, preview, pickerLabel, value, accept, uploadLabel, choice } = props;
  return (
    <Flex direction="column" gap="md">
      <Text variant="label">{title}</Text>
      <Text variant="caption" tone="muted">{note}</Text>
      <Flex gap="md" align="center">
        {preview}
        <IconPicker
          label={pickerLabel}
          icons={BOARD_ICON_NAMES}
          allowUpload={false}
          disabled={choice.busy}
          value={value}
          onChange={(next) => {
            const name = next.kind === 'icon' ? toBoardIconName(next.name) : undefined;
            if (name) void choice.pickIcon(name);
          }}
        />
      </Flex>
      <DropZone variant="inline" size="sm" accept={accept} label={uploadLabel} disabled={choice.busy} onDrop={(files) => void choice.upload(files)} onReject={choice.reject} />
      <AvatarReset disabled={choice.busy || choice.setting === null} problem={choice.problem} onReset={() => void choice.reset()} />
    </Flex>
  );
};

export { LookPicker };
