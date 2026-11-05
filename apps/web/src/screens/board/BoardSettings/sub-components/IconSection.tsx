/* @layer renderer-app @kind component */
import { Button, DropZone, Flex, Pressable, SegmentedControl, Text } from '@drizztdourden08/tessera';
import { BRAND_FAMILY, BrandMark } from '@drizztdourden08/tessera/brand';
import { useState } from 'react';
import { AVATAR_BRANDS, BRAND_PREFIX } from '../../../../avatar';
import { BoardIcon } from '../../../../shell/BoardIcon';
import { useIconChoice } from '../behavior/useIconChoice';
import { ICON_GRID_NOTE, ICON_MODES, ICON_NOTE, ICON_TYPES } from '../BoardSettings.constants';
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
        <Flex gap="sm" align="center" wrap role="group" aria-label="Icon brand marks">
          {AVATAR_BRANDS.map((app) => (
            <Pressable
              key={app}
              aria-label={`Use the ${BRAND_FAMILY[app].name} mark as the icon`}
              aria-pressed={choice.setting === `${BRAND_PREFIX}${app}`}
              disabled={choice.busy}
              onClick={() => void choice.pickBrand(app)}
            >
              <BrandMark app={app} size="lg" variant="app-icon" title={BRAND_FAMILY[app].name} />
            </Pressable>
          ))}
        </Flex>
      )}
      {mode === 'icon' && <Text variant="caption" tone="muted">{ICON_GRID_NOTE}</Text>}
      {mode === 'upload' && (
        <DropZone variant="inline" size="sm" accept={ICON_TYPES} label="Upload a PNG or SVG" disabled={choice.busy} onDrop={(files) => void choice.upload(files)} />
      )}
      {choice.problem && <Text variant="caption" tone="danger" role="alert">{choice.problem}</Text>}
    </Flex>
  );
};

export { IconSection };
