/* @layer renderer-app @kind logic */
import { useStored } from '../../../shared';
import { LAYOUT_KEY } from '../tree.constants';
import type { KanbanLayout, LayoutState } from '../tree.type';

const useLayout = (): LayoutState => {
  const [stored, store] = useStored(LAYOUT_KEY);
  const layout: KanbanLayout = stored === 'tree' ? 'tree' : 'columns';
  return { layout, setLayout: store };
};

export { useLayout };
