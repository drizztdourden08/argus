/* @layer renderer-app @kind component */
import { Flex, Text } from '@drizztdourden08/tessera';
import type { VersionFilesProps } from '../releases.type';
import { FileLine } from './FileLine';

const VersionFiles = (props: VersionFilesProps) => {
  const { version } = props;
  const { assets, packages } = version;
  if (assets.length === 0 && packages.length === 0) return <Text variant="caption" tone="muted">No files in this version.</Text>;
  return (
    <Flex direction="column" gap="xs">
      {assets.map((asset) => <FileLine key={`asset-${asset.name}`} name={asset.name} url={asset.url} size={asset.size} kind={asset.kind} />)}
      {packages.map((pack) => <FileLine key={`package-${pack.name}@${pack.version}`} name={`${pack.name}@${pack.version}`} url={pack.url} size={pack.size} kind="package" />)}
    </Flex>
  );
};

export { VersionFiles };
