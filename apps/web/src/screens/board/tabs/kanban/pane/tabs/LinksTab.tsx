/* @layer renderer-app @kind component */
import { Flex } from '@drizztdourden08/tessera';
import { DependencySection } from '../../sub-components/DependencySection';
import { BlockedSection } from '../BlockedSection';
import { TrailSection } from '../trail/TrailSection';
import type { PaneBodyProps } from '../pane.type';

const LinksTab = (props: PaneBodyProps) => {
  const { data } = props;
  return (
    <Flex direction="column" gap="lg">
      <DependencySection item={data.item} now={data.now} />
      <BlockedSection data={data} />
      <TrailSection data={data} />
    </Flex>
  );
};

export { LinksTab };
