/* @layer renderer-app @kind component */
import { Box, Callout, EmptyState, Flex, Text } from '@drizztdourden08/tessera';
import { useCallback } from 'react';
import type { Item } from '../../../../data/kanban';
import { useCollection } from '../../../../data/react';
import { readBoardReleases, REMOTE_POLL_MS, useRemoteRead } from '../../../../data/remote';
import type { PullRequest } from '../../../../data/types';
import { Panel } from '../../../../shell';
import { ChannelList } from '../../../releases';
import { useTabView } from '../../BoardTabsRow';
import { viewChannels } from './model/releases-view';
import { EMPTY_HINT, EMPTY_TEXT, LOADING_TEXT, NO_MATCH_TEXT, RELEASES_SETUP, RELEASES_TAB } from './releases.constants';
import type { ReleasesTabProps } from './releases.type';
import { ReleasesTools } from './ReleasesTools';

const countedText = (shown: number, total: number): string => {
  if (total === 0) return '';
  return `${shown === total ? total : `${shown} of ${total}`} ${total === 1 ? 'channel' : 'channels'}`;
};

const ReleasesTab = (props: ReleasesTabProps) => {
  const { boardKey } = props;
  const load = useCallback(() => readBoardReleases(boardKey), [boardKey]);
  const { data, error } = useRemoteRead(load, REMOTE_POLL_MS);
  const items = useCollection<Item>(boardKey, 'items');
  const prs = useCollection<PullRequest>(boardKey, 'prs');
  const view = useTabView(RELEASES_TAB, RELEASES_SETUP);
  if (error && !data) return <Callout tone="danger">{`The releases could not be read: ${error}.`}</Callout>;
  if (!data) return <Text tone="muted">{LOADING_TEXT}</Text>;
  const shown = viewChannels(data, view.filters, view.order, view.direction);
  return (
    <>
      <ReleasesTools boardKey={boardKey} view={view} channels={data} />
      <Panel title="Releases" note={countedText(shown.length, data.length) || undefined}>
        {shown.length > 0 && (
          <Flex direction="column" gap="lg">
            {shown.map((channel) => (
              <Box key={channel.channel} data-swap-item>
                <ChannelList channels={[channel]} project={boardKey} prefixed={false} shipped={{ items: items.docs, prs: prs.docs }} />
              </Box>
            ))}
          </Flex>
        )}
        {shown.length === 0 && <EmptyState message={data.length > 0 ? NO_MATCH_TEXT : EMPTY_TEXT} hint={data.length > 0 ? undefined : EMPTY_HINT} />}
      </Panel>
    </>
  );
};

export { ReleasesTab };
