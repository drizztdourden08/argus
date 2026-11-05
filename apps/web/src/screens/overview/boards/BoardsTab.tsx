/* @layer renderer-app @kind component */
import { AsideLayout, ContentSwap, EmptyState, Stack } from '@drizztdourden08/tessera';
import { useMemo, useState } from 'react';
import { useRemoteSyncs } from '../../../data/remote';
import { RunLogDrawer } from '../../board/run-terminal';
import { useNow } from '../clock';
import { useRunningRuns } from './behavior/useRunningRuns';
import type { LogTarget } from './boards.type';
import { arrangeEntries } from './behavior/arrange-entries';
import { groupEntries } from './behavior/group-entries';
import { NO_BOARDS, NO_BOARDS_YET } from './boards-view.constants';
import type { BoardsTabProps } from './boards-view.type';
import { BoardsGrid } from './BoardsGrid';
import { BoardsRows } from './BoardsRows';
import { NowRail } from './NowRail';
import { RAIL_BREAKPOINT, RAIL_LABEL } from './boards-view.constants';

const BoardsTab = (props: BoardsTabProps) => {
  const { data, view, send, onRail } = props;
  const { search, sort, group, layout } = view;
  const syncs = useRemoteSyncs();
  const now = useNow();
  const [log, setLog] = useState<LogTarget | null>(null);
  const groups = useMemo(() => groupEntries(arrangeEntries(data.entries, search, sort), group), [data.entries, search, sort, group]);
  const List = layout === 'cards' ? BoardsGrid : BoardsRows;
  const empty = data.ready && data.entries.length === 0 ? NO_BOARDS_YET : NO_BOARDS;
  return (
    <Stack gap="md">
      <AsideLayout
        main={
          <ContentSwap swapKey={`${layout}|${group}|${sort.key}|${sort.direction}|${search}`} effect="reflow">
            {groups.length === 0
              ? <EmptyState message={empty} />
              : <List groups={groups} now={now} send={send} syncs={syncs} onLog={(key, run) => setLog({ key, run })} />}
          </ContentSwap>
        }
        aside={<NowRail entries={data.entries} />}
        asideWidth="md"
        breakpoint={RAIL_BREAKPOINT}
        asideLabel={RAIL_LABEL}
        onAsideChange={onRail}
      />
      <RunLogDrawer run={log?.run ?? null} boardKey={log?.key ?? ''} onClose={() => setLog(null)} />
    </Stack>
  );
};

export { BoardsTab };
