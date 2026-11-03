/* @layer renderer-app @kind types */
import type { ItemRef } from '../../data/kanban';
import type { WithId } from '../../data/react';
import type { ReleaseChannel, ReleasePr, ReleaseVersion } from '../../data/remote';
import type { PullRequest } from '../../data/types';

type PullRequestDoc = WithId<PullRequest>;

interface ShippedSource {
  items: readonly ItemRef[];
  prs: readonly PullRequestDoc[];
}

interface ReleaseScope {
  project: string;
  prefixed: boolean;
  shipped?: ShippedSource;
}

interface Shipped {
  items: ItemRef[];
  unknown: string[];
  prs: { pr: ReleasePr; local: PullRequestDoc | null }[];
}

interface ChannelListProps extends ReleaseScope {
  channels: readonly ReleaseChannel[];
}

interface ChannelSectionProps extends ReleaseScope {
  channel: ReleaseChannel;
}

interface VersionCardProps extends ReleaseScope {
  channel: string;
  version: ReleaseVersion;
  latest: boolean;
}

interface VersionFootProps {
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

interface ShippedRowsProps {
  version: ReleaseVersion;
  project: string;
  source: ShippedSource;
}

interface ShippedPrRowProps {
  boardKey: string;
  pr: ReleasePr;
  local: PullRequestDoc | null;
}

export type {
  ChannelListProps, ChannelSectionProps, FileLineProps, PullRequestDoc, Shipped, ShippedPrRowProps, ShippedRowsProps, ShippedSource, VersionCardProps, VersionFilesProps, VersionFootProps,
};
