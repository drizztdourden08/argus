/* @layer renderer-app @kind component */
import { Card, Flex, Link, Tag, Text } from '@drizztdourden08/tessera';
import { shortCommit } from '../../../format';
import { Age, ReplayTag } from '../../../shell';
import type { VersionCardProps } from '../releases.type';
import { VersionFiles } from './VersionFiles';

const VersionCard = (props: VersionCardProps) => {
  const { channel, version, latest } = props;
  return (
    <Card data-version={version.version} role="group" aria-label={`Release ${version.version} of channel ${channel}`} edge="border" edgeTone={latest ? 'success' : undefined}>
      <Flex direction="column" gap="sm">
        <Flex gap="sm" align="center" wrap>
          <Text weight={600} numeric>{version.version}</Text>
          {latest && <Tag variant="urgency" color="success" title="The version the update feed points at">latest</Tag>}
          {version.title && <Text>{version.title}</Text>}
        </Flex>
        <Flex gap="md" align="center" wrap>
          {version.tag && <Tag icon="tag" title="Git tag">{version.tag}</Tag>}
          <Text variant="caption" tone="muted" title="Commit">{'commit '}<Text variant="caption" tone="dim" mono>{shortCommit(version.commit)}</Text></Text>
          <Text variant="caption" tone="muted">{'published '}<Age value={version.publishedAt} ago /></Text>
          {version.notesUrl && <Link href={version.notesUrl} title="Open the release notes">Release notes</Link>}
          <ReplayTag replay={version.replay} />
        </Flex>
        <VersionFiles version={version} />
      </Flex>
    </Card>
  );
};

export { VersionCard };
