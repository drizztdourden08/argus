/* @layer renderer-app @kind logic */
import { useCallback, useMemo, useRef, useState } from 'react';
import { useCollection } from '../../../../data/react';
import type { Wake } from '../../../../data/types';
import type { NoticesView } from '../BoardHeader.type';
import { passiveNotices } from './passive-notices';

const useNotices = (boardKey: string): NoticesView => {
  const wakes = useCollection<Wake>(boardKey, 'wakes');
  const notices = useMemo(() => passiveNotices(wakes.docs), [wakes.docs]);
  const anchor = useRef<HTMLButtonElement>(null);
  const [open, setOpen] = useState(false);
  const toggle = useCallback(() => setOpen((was) => !was), []);
  const close = useCallback(() => setOpen(false), []);
  return { notices, anchor, open, toggle, close };
};

export { useNotices };
