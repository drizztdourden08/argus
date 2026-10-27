/* @layer renderer-app @kind component */
import { Box, SectionHeader } from '@drizztdourden08/tessera';
import type { DrawerSectionProps } from '../AgentsTab.type';

const DrawerSection = (props: DrawerSectionProps) => {
  const { title, children } = props;
  return (
    <Box as="section" aria-label={title}>
      <SectionHeader title={title} level={4} />
      {children}
    </Box>
  );
};

export { DrawerSection };
