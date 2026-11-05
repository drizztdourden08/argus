/* @layer renderer-app @kind logic */
import { useMemo } from 'react';
import { PORTFOLIO_SCOPE, useCollection } from '../../../../data/react';
import type { SentEntry } from '../../../../data/types';
import { SENT_LIMIT } from '../boards-send.constants';
import type { SentItem } from '../model';

const useSentDown = (): { total: number; rows: SentItem[] } => {
  const { docs } = useCollection<SentEntry>(PORTFOLIO_SCOPE, 'entries');
  const rows = useMemo(
    () => [...docs].sort((a, b) => String(b.createdAt).localeCompare(String(a.createdAt))).slice(0, SENT_LIMIT),
    [docs],
  );
  return { total: docs.length, rows };
};

export { useSentDown };
