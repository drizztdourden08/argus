/* @layer renderer-app @kind component */
import { Flex, Text } from '@drizztdourden08/tessera';
import './OverviewHeader.css';

const OverviewHeader = () => (
  <Flex as="header" className="overview-header" align="center" gap="md">
    <Text as="h1" className="overview-header__title">Overview</Text>
  </Flex>
);

export { OverviewHeader };
