/* @layer renderer-app @kind types */
import type { ReleaseChannel, ReleaseVersion } from '../../data/remote';

interface ChannelListProps {
  channels: readonly ReleaseChannel[];
}

interface ChannelSectionProps {
  channel: ReleaseChannel;
}

interface VersionCardProps {
  channel: string;
  version: ReleaseVersion;
  latest: boolean;
}

interface VersionFilesProps {
  version: ReleaseVersion;
}

interface FileLineProps {
  name: string;
  url: string;
  size: number;
  kind?: string;
}

export type { ChannelListProps, ChannelSectionProps, FileLineProps, VersionCardProps, VersionFilesProps };
