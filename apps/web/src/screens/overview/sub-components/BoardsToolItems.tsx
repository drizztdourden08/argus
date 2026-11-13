/* @layer renderer-app @kind logic */
import { GroupControl, SearchInput, SegmentedControl, SortControl } from '@drizztdourden08/tessera';
import type { OverflowRowItem } from '@drizztdourden08/tessera';
import type { OverviewView } from '../boards/boards-view.type';
import { GROUP_OPTIONS, LAYOUT_OPTIONS, SORT_OPTIONS } from '../overview.constants';
import { groupFrom } from '../behavior/group-from';

const boardsToolItems = (view: OverviewView): OverflowRowItem[] => [
  { id: 'search', priority: 4, node: <SearchInput size="sm" value={view.search} onChange={view.setSearch} placeholder="Search boards" aria-label="Search boards" /> },
  {
    id: 'group',
    priority: 3,
    node: <GroupControl options={GROUP_OPTIONS} value={view.group === 'none' ? null : view.group} onChange={(id) => view.setGroup(groupFrom(id))} />,
  },
  { id: 'sort', priority: 3, node: <SortControl options={SORT_OPTIONS} value={view.sort} onChange={view.setSort} /> },
  {
    id: 'layout',
    priority: 2,
    end: true,
    node: <SegmentedControl selectedTone="neutral" size="sm" aria-label="Boards layout" value={view.layout} options={LAYOUT_OPTIONS} onChange={view.setLayout} />,
  },
];

export { boardsToolItems };
