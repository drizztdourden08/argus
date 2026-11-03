/* @layer renderer-app @kind component */
import { Box, Flex, Tag, Text } from '@drizztdourden08/tessera';
import { RecordGroup } from '../../../shell';
import { sortVersions } from '../model';
import type { ChannelSectionProps } from '../releases.type';
import { VersionCard } from './VersionCard';

const ChannelSection = (props: ChannelSectionProps) => {
  const { channel, ...scope } = props;
  return (
    <Box data-channel={channel.channel}>
      <RecordGroup
        header={(
          <Flex gap="sm" align="center">
            <Text variant="subtitle">{channel.channel}</Text>
            {channel.latest && <Tag title="The version the update feed points at">{`latest ${channel.latest}`}</Tag>}
          </Flex>
        )}
        count={channel.versions.length}
        label={`Channel ${channel.channel}`}
      >
        {sortVersions(channel.versions).map((version) => (
          <VersionCard key={version.version} channel={channel.channel} version={version} latest={version.version === channel.latest} {...scope} />
        ))}
      </RecordGroup>
    </Box>
  );
};

export { ChannelSection };
