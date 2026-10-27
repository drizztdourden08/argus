/* @layer renderer-app @kind component */
import { Divider, Flex, SectionHeader } from '@drizztdourden08/tessera';
import type { DrawerSectionProps } from '../drawer-props.type';

const DrawerSection = (props: DrawerSectionProps) => {
  const { title, children } = props;
  return (
    <Flex as="section" direction="column" gap="sm" aria-label={title}>
      <Divider />
      <SectionHeader title={title} level={4} />
      {children}
    </Flex>
  );
};

export { DrawerSection };
