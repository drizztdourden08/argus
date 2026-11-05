/* @layer renderer-app @kind component */
import { Button, DropZone, Flex, IconPicker, SegmentedControl, Text } from '@drizztdourden08/tessera';
import { useState } from 'react';
import { AVATAR_BRANDS, BRAND_PREFIX } from '../../../../avatar';
import { BOARD_ICON_NAMES, toBoardIconName } from '../../../../board-icon';
import { BoardIcon } from '../../../../shell/BoardIcon';
import { BrandMarkPicker } from '../../../../shell/BrandMarkPicker';
import { pickerValue } from '../behavior/picker-value';
import { useIconChoice } from '../behavior/useIconChoice';
import { ICON_MODES, ICON_NOTE, ICON_PICKER_LABEL, ICON_TYPES } from '../BoardSettings.constants';
import type { IconMode, SectionProps } from '../BoardSettings.type';

const IconSection = (props: SectionProps) => {
  const { boardKey } = props;
  const choice = useIconChoice(boardKey);
  const [mode, setMode] = useState<IconMode>('brand');
  return (
    <Flex direction="column" gap="md">
      <Text variant="label">Board icon</Text>
      <Text variant="caption" tone="muted">{ICON_NOTE}</Text>
      <Flex gap="md" align="center">
        <BoardIcon boardKey={boardKey} icon={choice.setting} avatar={choice.avatar} size="lg" title="Board icon preview" />
        <Button size="sm" variant="secondary" disabled={choice.busy || choice.setting === null} onClick={() => void choice.reset()}>Use the default</Button>
      </Flex>
      <SegmentedControl selectedTone="neutral" size="sm" aria-label="Icon source" value={mode} options={ICON_MODES} onChange={setMode} />
      {mode === 'brand' && (
        <BrandMarkPicker
          label="Icon brand marks"
          value={AVATAR_BRANDS.find((app) => choice.setting === `${BRAND_PREFIX}${app}`) ?? null}
          disabled={choice.busy}
          onPick={(app) => void choice.pickBrand(app)}
        />
      )}
      {mode === 'icon' && (
        <IconPicker
          label={ICON_PICKER_LABEL}
          icons={BOARD_ICON_NAMES}
          allowUpload={false}
          disabled={choice.busy}
          value={pickerValue(choice.setting)}
          onChange={(value) => {
            const name = value.kind === 'icon' ? toBoardIconName(value.name) : undefined;
            if (name) void choice.pickIcon(name);
          }}
        />
      )}
      {mode === 'upload' && (
        <DropZone variant="inline" size="sm" accept={ICON_TYPES} label="Upload a PNG or SVG" disabled={choice.busy} onDrop={(files) => void choice.upload(files)} onReject={choice.reject} />
      )}
      {choice.problem && <Text variant="caption" tone="danger" role="alert">{choice.problem}</Text>}
    </Flex>
  );
};

export { IconSection };
