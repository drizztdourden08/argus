/* @layer renderer-app @kind component */
import { Box, SectionHeader } from '@drizztdourden08/tessera';
import type { DrawerSectionProps } from '../drawer-props.type';

const DrawerSection = (props: DrawerSectionProps) => {
  const { title, count, children } = props;
  return (
    <Box as="section" aria-label={title}>
      <SectionHeader title={title} count={count} hideZero={false} level={4} />
      {children}
    </Box>
  );
};

export { DrawerSection };
