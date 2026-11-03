/* @layer renderer-app @kind component */
import { Flex } from '@drizztdourden08/tessera';
import type { ChannelListProps } from './releases.type';
import { ChannelSection } from './sub-components/ChannelSection';

const ChannelList = (props: ChannelListProps) => {
  const { channels, ...scope } = props;
  return (
    <Flex direction="column" gap="lg">
      {channels.map((channel) => <ChannelSection key={channel.channel} channel={channel} {...scope} />)}
    </Flex>
  );
};

export { ChannelList };
