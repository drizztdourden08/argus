/* @layer renderer-app @kind types */
import type { SortState } from '@drizztdourden08/tessera';
import type { ReactNode } from 'react';
import type { SyncMap } from '../../../data/remote';
import type { Card } from '../../../data/types';
import type { LogOpener, SendDown } from './boards.type';
import type { BoardLive, BoardRecord } from './model';

type GroupMode = 'attention' | 'state' | 'none';

type BoardsLayout = 'cards' | 'rows';

type SortKey = 'attention' | 'name' | 'updated';

interface OverviewView {
  search: string;
  group: GroupMode;
  sort: SortState;
  layout: BoardsLayout;
  setSearch: (search: string) => void;
  setGroup: (group: GroupMode) => void;
  setSort: (sort: SortState) => void;
  setLayout: (layout: BoardsLayout) => void;
}

interface BoardEntry {
  record: BoardRecord;
  card: Card | null;
  live: BoardLive;
  updatedAt: string;
}

interface BoardEntries {
  ready: boolean;
  entries: BoardEntry[];
}

interface EntryGroup {
  id: string;
  label: string;
  folded: boolean;
  entries: BoardEntry[];
}

interface BoardsTotals {
  boards: number;
  running: number;
  needs: number;
}

interface BoardsTabProps {
  data: BoardEntries;
  view: OverviewView;
  send: SendDown;
  onRail: (shown: boolean) => void;
}

interface BoardsListProps {
  groups: EntryGroup[];
  now: number;
  send: SendDown;
  syncs: SyncMap | null;
  onLog: LogOpener;
}

interface BoardsGroupProps {
  group: EntryGroup;
  children: ReactNode;
}

interface BoardRowProps {
  entry: BoardEntry;
}

interface EntriesProps {
  entries: readonly BoardEntry[];
}

export type {
  BoardEntries, BoardEntry, BoardRowProps, BoardsGroupProps, BoardsLayout, BoardsListProps, BoardsTabProps, BoardsTotals, EntriesProps, EntryGroup, GroupMode, OverviewView, SortKey,
};
