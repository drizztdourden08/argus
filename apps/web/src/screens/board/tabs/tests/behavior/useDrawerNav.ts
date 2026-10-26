/* @layer renderer-app @kind logic */
import { useCallback, useEffect, useState } from 'react';
import { TYPING_SELECTOR } from '../tests.constants';
import type { DrawerNav } from '../tests.type';

const onScreenIds = (): string[] => {
  const ids: string[] = [];
  for (const row of document.querySelectorAll<HTMLElement>('[data-open-test]')) {
    const id = row.dataset.openTest;
    if (id && !ids.includes(id) && !row.closest('details:not([open])')) ids.push(id);
  }
  return ids;
};

const useDrawerNav = (selected: string | null, open: (testId: string) => void): DrawerNav => {
  const [list, setList] = useState<string[]>([]);
  useEffect(() => {
    if (!selected) return;
    setList((before) => {
      const here = onScreenIds();
      if (here.includes(selected)) return here;
      return before.includes(selected) ? before : [selected];
    });
  }, [selected]);

  const at = selected ? list.indexOf(selected) : -1;
  const previous = at > 0 ? list[at - 1] ?? null : null;
  const next = at >= 0 ? list[at + 1] ?? null : null;
  const step = useCallback((direction: -1 | 1) => {
    const target = direction === 1 ? next : previous;
    if (target) open(target);
  }, [next, previous, open]);

  useEffect(() => {
    if (!selected) return undefined;
    const onKey = (event: KeyboardEvent) => {
      if ((event.key !== '[' && event.key !== ']') || event.ctrlKey || event.metaKey || event.altKey) return;
      if (event.target instanceof Element && event.target.matches(TYPING_SELECTOR)) return;
      event.preventDefault();
      step(event.key === ']' ? 1 : -1);
    };
    document.addEventListener('keydown', onKey);
    return () => document.removeEventListener('keydown', onKey);
  }, [selected, step]);

  return { previous, next, step };
};

export { useDrawerNav };
