/* @layer renderer-app @kind component */
import { Flex, Text } from '@drizztdourden08/tessera';
import { LEAD_TEXT } from '../builds.constants';
import type { BuildsLeadProps } from '../builds-props.type';

const BuildsLead = (props: BuildsLeadProps) => {
  const { counts } = props;
  return (
    <Flex as="span" gap="sm" wrap>
      {counts && <Text variant="caption" tone="muted">{counts}</Text>}
      <Text variant="caption" tone="dim">{LEAD_TEXT}</Text>
    </Flex>
  );
};

export { BuildsLead };
