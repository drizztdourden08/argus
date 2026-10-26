/* @layer renderer-app @kind component */
import { Flex, SectionHeader } from '@drizztdourden08/tessera';
import type { ThreadSectionProps } from '../ThreadDrawer.type';

const ThreadSection = (props: ThreadSectionProps) => {
  const { title, count, children } = props;
  return (
    <Flex direction="column" gap="xs" role="group" aria-label={title}>
      <SectionHeader title={title} count={count} hideZero={false} level={4} />
      {children}
    </Flex>
  );
};

export { ThreadSection };
