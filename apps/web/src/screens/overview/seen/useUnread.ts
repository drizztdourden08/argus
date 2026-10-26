/* @layer renderer-app @kind logic */
import { useEffect, useMemo, useRef, useState } from 'react';
import { isNewer } from './is-newer';
import { newestAt } from './newest-at';
import { readStored } from './read-stored';
import type { Timed, Unread } from './seen.type';
import { writeStored } from './write-stored';

const useUnread = <T extends Timed>(storageKey: string, items: readonly T[], active: boolean): Unread<T> => {
  const seenRef = useRef(readStored(storageKey));
  const [seen, setSeen] = useState(seenRef.current);
  const [freshSince, setFreshSince] = useState(Number.NaN);
  const wasActive = useRef(false);
  const newest = newestAt(items);

  useEffect(() => {
    if (!active) {
      wasActive.current = false;
      return;
    }
    if (!wasActive.current) setFreshSince(seenRef.current);
    wasActive.current = true;
    const next = Math.max(Date.now(), newest);
    seenRef.current = next;
    setSeen(next);
    writeStored(storageKey, next);
  }, [active, newest, storageKey]);

  const unread = useMemo(() => (active ? [] : items.filter((item) => isNewer(item.at, seen))), [active, items, seen]);
  return { unread, freshSince };
};

export { useUnread };
