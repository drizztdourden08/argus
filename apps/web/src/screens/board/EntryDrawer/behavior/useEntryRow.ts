/* @layer renderer-app @kind logic */
import { useMemo } from 'react';
import { useCollection } from '../../../../data/react';
import type { Entry } from '../../../../data/types';
import { entryOf } from '../../tabs/kanban/record';
import type { EntryRow } from '../../tabs/kanban/record';

const useEntryRow = (boardKey: string, id: string | null): EntryRow | null => {
  const entries = useCollection<Entry>(boardKey, 'entries');
  return useMemo(() => entryOf(id, entries.docs), [id, entries.docs]);
};

export { useEntryRow };
