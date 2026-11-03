/* @layer renderer-app @kind component */
import { Flex, Link, Spacer, Tag, Text } from '@drizztdourden08/tessera';
import { shortCommit } from '../../../format';
import { Age, ReplayTag } from '../../../shell';
import type { VersionFootProps } from '../releases.type';

const countText = (count: number, noun: string): string => `${count} ${noun}${count === 1 ? '' : 's'}`;

const VersionFoot = (props: VersionFootProps) => {
  const { version, latest } = props;
  return (
    <Flex gap="sm" align="center" wrap>
      {latest && <Tag variant="urgency" color="success" title="The version the update feed points at">latest</Tag>}
      {version.packages.length > 0 && <Tag title="Packages in this version">{countText(version.packages.length, 'package')}</Tag>}
      {version.assets.length > 0 && <Tag title="App files in this version">{countText(version.assets.length, 'file')}</Tag>}
      {version.tag && <Tag icon="tag" title="Git tag">{version.tag}</Tag>}
      <Text variant="caption" tone="muted" title="Commit"><Text variant="caption" tone="dim" mono>{shortCommit(version.commit)}</Text></Text>
      {version.notesUrl && <Link href={version.notesUrl} title="Open the release notes">Release notes</Link>}
      <ReplayTag replay={version.replay} />
      <Spacer />
      <Text variant="caption" tone="muted"><Age value={version.publishedAt} /></Text>
    </Flex>
  );
};

export { VersionFoot };
