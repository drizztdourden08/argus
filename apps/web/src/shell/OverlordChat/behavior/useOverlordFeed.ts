/* @layer renderer-app @kind logic */
import { useContext } from 'react';
import type { OverlordFeedApi } from '../OverlordChat.type';
import { OverlordFeedContext } from './overlord-feed-context';

const useOverlordFeed = (): OverlordFeedApi => {
  const api = useContext(OverlordFeedContext);
  if (!api) throw new Error('useOverlordFeed needs an OverlordFeedProvider.');
  return api;
};

export { useOverlordFeed };
