/* @layer renderer-app @kind logic */
import { useMemo } from 'react';
import { PORTFOLIO_SCOPE, useCollection } from '../../../../data/react';
import type { NotificationDoc } from '../../../../data/types';
import { sortNotes } from '../model';
import type { NoteItem } from '../model';

const useNotificationFeed = (): NoteItem[] => {
  const { docs } = useCollection<NotificationDoc>(PORTFOLIO_SCOPE, 'notifications');
  return useMemo(() => sortNotes(docs), [docs]);
};

export { useNotificationFeed };
