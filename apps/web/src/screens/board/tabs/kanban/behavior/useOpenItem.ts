/* @layer renderer-app @kind logic */
import { useCallback } from 'react';
import { useLocation, useNavigate } from 'react-router';
import type { ActionDoc } from '../../../../../data/kanban';
import type { WithId } from '../../../../../data/react';
import { itemHash } from '../model';
import { KANBAN_HASH } from '../kanban.constants';
import type { OpenItem } from '../kanban.type';

const resolved = (id: string | null, actions: readonly WithId<ActionDoc>[]): string | null => {
  if (!id?.startsWith('H-')) return id;
  return actions.find((action) => action.id === id)?.item ?? null;
};

const useOpenItem = (actions: readonly WithId<ActionDoc>[]): OpenItem => {
  const { hash, search } = useLocation();
  const navigate = useNavigate();
  const go = useCallback((next: string) => void navigate({ search, hash: next }, { replace: true }), [navigate, search]);
  const open = useCallback((id: string) => go(id), [go]);
  const close = useCallback(() => go(KANBAN_HASH), [go]);
  return { id: resolved(itemHash(hash), actions), open, close };
};

export { useOpenItem };
