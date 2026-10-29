/* @layer renderer-app @kind component */
import { BrandMark, Flex, Text } from '@drizztdourden08/tessera';
import type { OverviewHeaderProps } from '../overview.type';
import { ProjectSwitcher } from './ProjectSwitcher';
import './OverviewHeader.css';

const OverviewHeader = (props: OverviewHeaderProps) => {
  const { cards } = props;
  return (
    <Flex as="header" className="overview-header" align="center" justify="between" gap="md">
      <Flex className="overview-header__brand" align="center" gap="sm">
        <BrandMark app="argus" size="md" />
        <Text as="h1" className="overview-header__title">Argus</Text>
      </Flex>
      <Flex className="overview-header__tools" align="center"><ProjectSwitcher cards={cards} /></Flex>
    </Flex>
  );
};

export { OverviewHeader };
