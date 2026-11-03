/* @layer renderer-app @kind logic */
import { useCallback } from 'react';
import { useLocation, useNavigate } from 'react-router';
import type { ItemRef } from '../../../../../../data/kanban';
import { useStored } from '../../../../shared';
import { hashTab, typeOf } from '../../model';
import { TAB_STORE } from '../model/pane-model.constants';

const usePaneTab = (item: ItemRef, tabs: readonly string[]): [string, (tab: string) => void] => {
  const { hash, search } = useLocation();
  const navigate = useNavigate();
  const [saved, save] = useStored(`${TAB_STORE}.${typeOf(item)}`);
  const wanted = [hashTab(hash), saved].find((tab) => tab !== null && tab !== '' && tabs.includes(tab));
  const select = useCallback(
    (tab: string) => {
      save(tab);
      void navigate({ search, hash: `${item.id}/${tab}` }, { replace: true });
    },
    [item.id, navigate, save, search],
  );
  return [wanted ?? tabs[0] ?? 'overview', select];
};

export { usePaneTab };
