/* @layer renderer-app @kind component */
import { Flex, Inline, Tag, Text } from '@drizztdourden08/tessera';
import { useRegistry } from '../behavior/useRegistry';
import { BAD_LOOK, INFO_LOOK } from '../dashboards.constants';
import { RegistryChipLink } from './RegistryChipLink';

const RegistryStrip = () => {
  const rows = useRegistry();
  if (rows.length === 0) return null;
  const waking = rows.filter((row) => row.state === 'waking').length;
  const faults = rows.filter((row) => row.state === 'fault').length;
  return (
    <Inline gap="md" align="center" wrap role="status" aria-label="Registered leads">
      <Text variant="label">{`Registered ${rows.length}`}</Text>
      {waking > 0 && <Tag {...INFO_LOOK}>{`${waking} waking`}</Tag>}
      {faults > 0 && <Tag {...BAD_LOOK}>{`${faults} fault${faults > 1 ? 's' : ''}`}</Tag>}
      <Flex gap="md" wrap>
        {rows.map((row) => <RegistryChipLink key={row.id} row={row} />)}
      </Flex>
    </Inline>
  );
};

export { RegistryStrip };
