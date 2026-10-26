/* @layer renderer-app @kind component */
import { Flex, SectionHeader } from '@drizztdourden08/tessera';
import type { DrawerSectionProps } from '../tests-drawer-props.type';

const DrawerSection = (props: DrawerSectionProps) => {
  const { title, count, children } = props;
  return (
    <Flex direction="column" gap="xs" role="group" aria-label={title}>
      <SectionHeader title={title} count={count} level={4} />
      {children}
    </Flex>
  );
};

export { DrawerSection };
