/* @layer renderer-app @kind logic */
import { useEffect } from 'react';
import { useLocation } from 'react-router';
import { queryString, scopePath } from '../../../data/api';
import { newTabId } from './new-tab-id';
import { presenceThread } from './presence-thread';

const usePresence = (boardKey: string): void => {
  const { search } = useLocation();
  const thread = presenceThread(search);
  useEffect(() => {
    if (!thread || typeof EventSource === 'undefined') return undefined;
    const source = new EventSource(`${scopePath(boardKey, 'presence')}${queryString({ thread, tab: newTabId() })}`);
    return () => source.close();
  }, [boardKey, thread]);
};

export { usePresence };
