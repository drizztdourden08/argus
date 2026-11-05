/* @layer renderer-app @kind types */
import type { ComponentType, ReactNode } from 'react';
import type { Agent } from '../../../../data/agents';
import type { BoardItem } from '../../../../data/boards';
import type { DepDoc } from '../../../../data/deps';
import type { ItemRef } from '../../../../data/kanban';
import type { WithId } from '../../../../data/react';
import type { Card, Launcher, Test } from '../../../../data/types';
import type { Worktree } from '../../../../data/worktrees';
import type { BoardRecord } from '../../../../screens/overview/boards/model';
import type { RecordRef } from '../../records.type';

type BoardField = 'goal' | 'note';

interface BoardLive {
  items: ItemRef[];
  agents: WithId<Agent>[];
  tests: WithId<Test>[];
  worktrees: WithId<Worktree>[];
  launchers: WithId<Launcher>[];
  deps: WithId<DepDoc>[];
}

interface BoardPaneData extends BoardLive {
  record: BoardRecord;
  card: Card | null;
  code: string;
  now: number;
}

interface BoardLookup {
  item: BoardItem | null;
  ready: boolean;
}

interface BoardPaneProps {
  target: RecordRef;
}

interface BoardFrameProps {
  item: BoardItem;
}

interface BoardTabProps {
  data: BoardPaneData;
}

interface BoardTab {
  label: string;
  Body: ComponentType<BoardTabProps>;
}

interface BoardTextEditorProps {
  record: BoardRecord;
  field: BoardField;
}

interface BoardArchiveToolProps {
  record: BoardRecord;
}

interface BoardFact {
  label: string;
  text: string;
}

interface LaneGroup {
  lane: string;
  label: string;
  items: ItemRef[];
}

interface DepSides {
  incoming: WithId<DepDoc>[];
  outgoing: WithId<DepDoc>[];
}

interface BoardRowProps {
  children: ReactNode;
}

interface BoardListProps {
  title: string;
  empty: string;
  count: number;
  children: ReactNode;
}

export type {
  BoardArchiveToolProps, BoardFact, BoardField, BoardFrameProps, BoardListProps, BoardLive, BoardLookup, BoardPaneData, BoardPaneProps, BoardRowProps, BoardTab, BoardTabProps,
  BoardTextEditorProps, DepSides, LaneGroup,
};
