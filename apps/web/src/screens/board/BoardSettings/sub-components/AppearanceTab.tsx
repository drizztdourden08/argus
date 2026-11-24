/* @layer renderer-app @kind component */
import { Divider, Flex } from '@drizztdourden08/tessera';
import type { SectionProps } from '../BoardSettings.type';
import { AvatarSection } from './AvatarSection';
import { IconSection } from './IconSection';

const AppearanceTab = (props: SectionProps) => (
  <Flex direction="column" gap="lg">
    <IconSection boardKey={props.boardKey} />
    <Divider />
    <AvatarSection boardKey={props.boardKey} />
  </Flex>
);

export { AppearanceTab };
