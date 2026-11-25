/* @layer renderer-app @kind component */
import { Flex, Link, Tag, Text } from '@drizztdourden08/tessera';
import { shortCommit } from '../../../format';
import { ReplayTag } from '../../../shell';
import type { VersionFootProps } from '../releases.type';

const countText = (count: number, noun: string): string => `${count} ${noun}${count === 1 ? '' : 's'}`;

const VersionFoot = (props: VersionFootProps) => {
  const { version, layout } = props;
  const compact = layout === 'compact';
  return (
    <Flex gap="sm" align="center" wrap>
      {version.packages.length > 0 && <Tag title="Packages in this version">{countText(version.packages.length, 'package')}</Tag>}
      {version.assets.length > 0 && <Tag title="App files in this version">{countText(version.assets.length, 'file')}</Tag>}
      {version.tag && version.tag !== `v${version.version}` && <Tag icon="tag" title="Git tag">{version.tag}</Tag>}
      {!compact && <Text variant="caption" tone="muted" title="Commit"><Text variant="caption" tone="dim" mono>{shortCommit(version.commit)}</Text></Text>}
      {!compact && version.notesUrl && <Text as="span" variant="caption"><Link href={version.notesUrl} title="Open the release notes">Release notes</Link></Text>}
      <ReplayTag replay={version.replay} />
    </Flex>
  );
};

export { VersionFoot };
