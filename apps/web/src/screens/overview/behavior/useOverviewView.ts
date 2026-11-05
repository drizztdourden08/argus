/* @layer renderer-app @kind logic */
import type { SortState } from '@drizztdourden08/tessera';
import { useState } from 'react';
import type { BoardsLayout, GroupMode, OverviewView } from '../boards/boards-view.type';
import { DEFAULT_SORT } from '../overview.constants';

const useOverviewView = (): OverviewView => {
  const [search, setSearch] = useState('');
  const [group, setGroup] = useState<GroupMode>('attention');
  const [sort, setSort] = useState<SortState>(DEFAULT_SORT);
  const [layout, setLayout] = useState<BoardsLayout>('cards');
  return { search, group, sort, layout, setSearch, setGroup, setSort, setLayout };
};

export { useOverviewView };
