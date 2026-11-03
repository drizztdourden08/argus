/* @layer renderer-app @kind logic */
import type { ReleaseChannel, ReleaseVersion } from '../../../../data/remote';
import type { PaneDoc } from '../pane.type';

const docOfVersion = (channel: string, version: ReleaseVersion): PaneDoc & { id: string } => ({
  id: version.version,
  channel,
  version: version.version,
  tag: version.tag,
  commit: version.commit,
  title: version.title,
  publishedAt: version.publishedAt,
  notesUrl: version.notesUrl ?? '',
  packages: version.packages.length,
  files: version.assets.length,
  items: version.items ?? [],
  prs: (version.prs ?? []).map((pr) => `#${pr.number}`),
});

const releaseDocs = (channels: readonly ReleaseChannel[]): (PaneDoc & { id: string })[] => {
  const byVersion = new Map<string, PaneDoc & { id: string }>();
  for (const channel of channels) {
    for (const version of channel.versions) {
      if (!byVersion.has(version.version)) byVersion.set(version.version, docOfVersion(channel.channel, version));
    }
  }
  return [...byVersion.values()];
};

export { releaseDocs };
