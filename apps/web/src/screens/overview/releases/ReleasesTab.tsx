/* @layer renderer-app @kind component */
import { Callout, EmptyState, Stack } from '@drizztdourden08/tessera';
import { readReleases, REMOTE_POLL_MS, useRemoteRead } from '../../../data/remote';
import { Panel } from '../../../shell';
import { ChannelList } from '../../releases';
import { ProjectLink } from '../projects';
import { EMPTY_TEXT, READING } from './releases.constants';

const ReleasesTab = () => {
  const { data, error } = useRemoteRead(readReleases, REMOTE_POLL_MS);
  if (!data) return error ? <Callout tone="danger">{`The releases could not be read: ${error}.`}</Callout> : <EmptyState message={READING} />;
  const shown = data.filter((entry) => entry.channels.length > 0);
  if (shown.length === 0) return <EmptyState message={EMPTY_TEXT} />;
  return (
    <Stack gap="md">
      {shown.map((entry) => (
        <Panel key={entry.project} title={<ProjectLink project={entry.project} hash="#releases" />}>
          <ChannelList channels={entry.channels} />
        </Panel>
      ))}
    </Stack>
  );
};

export { ReleasesTab };
