/* @layer renderer-app @kind component */
import { Flex, Text } from '@drizztdourden08/tessera';
import { LEAD_TEXT } from '../builds.constants';
import type { BuildsHeaderProps } from '../builds-props.type';
import { countsText } from '../model';

const BuildsHeader = (props: BuildsHeaderProps) => {
  const { views } = props;
  const counts = countsText(views);
  return (
    <Flex gap="sm" align="baseline" wrap>
      <Text variant="title">Builds</Text>
      {counts && <Text variant="caption" tone="muted">{counts}</Text>}
      <Text variant="caption" tone="muted">{LEAD_TEXT}</Text>
    </Flex>
  );
};

export { BuildsHeader };
