/* @layer renderer-app @kind component */
import { Flex, Text } from '@drizztdourden08/tessera';
import { ScreenshotGallery } from '../../screenshots';
import { imageGroups } from '../behavior/image-groups';
import type { EntryImagesProps } from '../OwnerFeed.type';

const EntryImages = (props: EntryImagesProps) => {
  const groups = imageGroups(props.entry);
  if (groups.length === 0) return null;
  return (
    <Flex direction="column" gap="xs">
      {groups.map((group) => (
        <Flex key={group.key} direction="column" gap="2xs">
          {group.title ? <Text variant="overline" tone="muted">{group.title}</Text> : null}
          <ScreenshotGallery shots={group.shots} />
        </Flex>
      ))}
    </Flex>
  );
};

export { EntryImages };
