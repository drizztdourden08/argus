/* @layer renderer-app @kind logic */
import { useRef, useState } from 'react';
import type { KeyboardEvent } from 'react';
import { MENU_KEY, TYPING_SELECTOR } from '../tests.constants';
import type { RowMenu } from '../tests.type';

const isMenuKey = (event: KeyboardEvent<HTMLElement>): boolean => {
  if (event.key.toLowerCase() !== MENU_KEY || event.altKey || event.ctrlKey || event.metaKey) return false;
  return !(event.target instanceof Element && (event.target.matches(TYPING_SELECTOR) || event.target.closest('[role="menu"]')));
};

const useRowMenu = (): RowMenu => {
  const [at, setAt] = useState<RowMenu['at']>(null);
  const opener = useRef<HTMLElement | null>(null);
  return {
    at,
    onKeyDown: (event) => {
      if (!isMenuKey(event)) return;
      event.preventDefault();
      opener.current = event.target instanceof HTMLElement ? event.target : null;
      const rect = event.currentTarget.getBoundingClientRect();
      setAt({ x: rect.left, y: rect.top });
    },
    close: () => {
      setAt(null);
      opener.current?.focus();
    },
  };
};

export { useRowMenu };
