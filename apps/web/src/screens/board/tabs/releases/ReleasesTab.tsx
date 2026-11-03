/* @layer renderer-app @kind component */
import { Callout, EmptyState, Text } from '@drizztdourden08/tessera';
import { useCallback } from 'react';
import type { Item } from '../../../../data/kanban';
import { useCollection } from '../../../../data/react';
import { readBoardReleases, REMOTE_POLL_MS, useRemoteRead } from '../../../../data/remote';
import type { PullRequest } from '../../../../data/types';
import { Panel } from '../../../../shell';
import { ChannelList } from '../../../releases';
import { EMPTY_HINT, EMPTY_TEXT, LOADING_TEXT } from './releases.constants';
import type { ReleasesTabProps } from './releases.type';

const ReleasesTab = (props: ReleasesTabProps) => {
  const { boardKey } = props;
  const load = useCallback(() => readBoardReleases(boardKey), [boardKey]);
  const { data, error } = useRemoteRead(load, REMOTE_POLL_MS);
  const items = useCollection<Item>(boardKey, 'items');
  const prs = useCollection<PullRequest>(boardKey, 'prs');
  if (error && !data) return <Callout tone="danger">{`The releases could not be read: ${error}.`}</Callout>;
  if (!data) return <Text tone="muted">{LOADING_TEXT}</Text>;
  return (
    <Panel title="Releases" note={data.length > 0 ? `${data.length} ${data.length === 1 ? 'channel' : 'channels'}` : undefined}>
      {data.length > 0
        ? <ChannelList channels={data} project={boardKey} prefixed={false} shipped={{ items: items.docs, prs: prs.docs }} />
        : <EmptyState message={EMPTY_TEXT} hint={EMPTY_HINT} />}
    </Panel>
  );
};

export { ReleasesTab };
