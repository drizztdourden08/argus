/* @layer renderer-app @kind logic */
import { useEffect } from 'react';
import { useLocation } from 'react-router';
import { useLink } from '../../../data/react';
import { presenceThread } from './presence-thread';

const usePresence = (boardKey: string): void => {
  const { search } = useLocation();
  const link = useLink();
  const thread = presenceThread(search);
  useEffect(() => (thread ? link.presence(boardKey, thread) : undefined), [link, boardKey, thread]);
};

export { usePresence };
