/* @layer renderer-app @kind logic */
import { useCallback } from 'react';
import { useStored } from '../../../../screens/board/shared';
import { useRecordPane } from '../../behavior/useRecordPane';
import { TAB_ALIASES, TAB_STORE } from '../model/pane-model.constants';

const usePaneTab = (kindId: string, tabs: readonly string[]): [string, (tab: string) => void] => {
  const pane = useRecordPane();
  const [saved, save] = useStored(`${TAB_STORE}.${kindId}`);
  const asked = pane.tab ? (TAB_ALIASES[pane.tab] ?? pane.tab) : null;
  const wanted = [asked, saved].find((tab) => tab !== null && tab !== '' && tabs.includes(tab));
  const select = useCallback(
    (tab: string) => {
      save(tab);
      pane.setTab(tab);
    },
    [pane, save],
  );
  return [wanted ?? tabs[0] ?? 'overview', select];
};

export { usePaneTab };
