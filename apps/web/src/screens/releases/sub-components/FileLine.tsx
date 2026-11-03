/* @layer renderer-app @kind component */
import { Flex, Icon, Link, Text } from '@drizztdourden08/tessera';
import { formatBytes } from '../../../format';
import type { FileLineProps } from '../releases.type';

const FileLine = (props: FileLineProps) => {
  const { name, url, size, kind } = props;
  return (
    <Flex gap="sm" align="center" wrap>
      <Icon name="download" size={12} aria-hidden />
      <Link href={url} download title={`Download ${name}`}>{name}</Link>
      <Text variant="caption" tone="dim" numeric>{formatBytes(size)}</Text>
      {kind && <Text variant="caption" tone="muted">{kind}</Text>}
    </Flex>
  );
};

export { FileLine };
