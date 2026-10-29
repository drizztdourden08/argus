/* @layer renderer-app @kind component */
import { Flex, Tag, Text } from '@drizztdourden08/tessera';
import { sortVersions } from '../model';
import type { ChannelSectionProps } from '../releases.type';
import { VersionCard } from './VersionCard';

const ChannelSection = (props: ChannelSectionProps) => {
  const { channel } = props;
  return (
    <Flex direction="column" gap="sm" role="group" aria-label={`Channel ${channel.channel}`}>
      <Flex gap="sm" align="center">
        <Text variant="subtitle">{channel.channel}</Text>
        {channel.latest && <Tag title="The version the update feed points at">{`latest ${channel.latest}`}</Tag>}
      </Flex>
      {sortVersions(channel.versions).map((version) => (
        <VersionCard key={version.version} channel={channel.channel} version={version} latest={version.version === channel.latest} />
      ))}
    </Flex>
  );
};

export { ChannelSection };
