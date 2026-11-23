/* @layer renderer-app @kind component */
import { Box, Flex } from '@drizztdourden08/tessera';
import type { TabPanelProps } from '../shared.type';
import { TabStats } from './TabStats';
import './TabPanel.css';

const TabPanel = (props: TabPanelProps) => {
  const { label, stats, tools, className, children } = props;
  return (
    <Box as="section" aria-label={label} className={className ? `tab-panel ${className}` : 'tab-panel'}>
      <Flex className="tab-panel__head" align="center" gap="sm">
        <TabStats stats={stats} label={`${label} counts`} />
        {tools}
      </Flex>
      {children}
    </Box>
  );
};

export { TabPanel };
