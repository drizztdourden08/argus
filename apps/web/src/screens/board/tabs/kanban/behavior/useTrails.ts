/* @layer renderer-app @kind logic */
import { useEffect, useState } from 'react';
import { apiRequest, scopePath } from '../../../../../data/api';
import type { ItemRef } from '../../../../../data/kanban';
import { blockedSignature, parseTrails } from '../model';
import type { TrailInfo } from '../model';
import { NO_TRAILS } from './useTrails.constants';

const useTrails = (boardKey: string, items: readonly ItemRef[]): ReadonlyMap<string, TrailInfo> => {
  const [trails, setTrails] = useState(NO_TRAILS);
  const signature = blockedSignature(items);

  useEffect(() => {
    let live = true;
    if (!signature) {
      setTrails(NO_TRAILS);
      return undefined;
    }
    apiRequest<unknown>(scopePath(boardKey, 'trails')).then(
      (raw) => live && setTrails(parseTrails(raw)),
      () => live && setTrails(NO_TRAILS),
    );
    return () => {
      live = false;
    };
  }, [boardKey, signature]);

  return trails;
};

export { useTrails };
