/* @layer renderer-app @kind logic */
import { useEffect, useState } from 'react';
import { apiRequest, scopePath } from '../../../../../../data/api';
import type { ItemRef } from '../../../../../../data/kanban';
import { blockedSignature } from '../../model';
import { parseTrail } from '../model/parse-trail';
import type { TrailTree } from '../pane.type';

const useTrail = (boardKey: string, item: ItemRef): TrailTree | null => {
  const [tree, setTree] = useState<TrailTree | null>(null);
  const signature = blockedSignature([item]);

  useEffect(() => {
    let live = true;
    setTree(null);
    if (!signature) return undefined;
    apiRequest<unknown>(scopePath(boardKey, 'items', item.id, 'trail')).then(
      (raw) => live && setTree(parseTrail(raw)),
      () => live && setTree(null),
    );
    return () => {
      live = false;
    };
  }, [boardKey, item.id, signature]);

  return tree;
};

export { useTrail };
