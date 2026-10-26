/* @layer renderer-app @kind logic */
import { useMemo } from 'react';
import type { ActionDoc } from '../../../../../data/kanban';
import { useCollection } from '../../../../../data/react';
import type { WithId } from '../../../../../data/react';
import type { Entry, LogDoc } from '../../../../../data/types';
import { itemRecord } from '../record';
import type { ItemRecord } from '../record';

const useItemRecord = (boardKey: string, itemId: string, actions: readonly WithId<ActionDoc>[]): ItemRecord => {
  const entries = useCollection<Entry>(boardKey, 'entries');
  const log = useCollection<LogDoc>(boardKey, 'log');
  return useMemo(
    () => itemRecord({ itemId, entries: entries.docs, actions, log: log.docs }),
    [itemId, entries.docs, actions, log.docs],
  );
};

export { useItemRecord };
