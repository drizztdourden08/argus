/* @layer renderer-app @kind component */
import type { ReactNode } from 'react';
import { OverlordFeedContext } from '../behavior/overlord-feed-context';
import { useOverlordFeedState } from '../behavior/useOverlordFeedState';

const OverlordFeedProvider = (props: { children: ReactNode }) => (
  <OverlordFeedContext.Provider value={useOverlordFeedState()}>{props.children}</OverlordFeedContext.Provider>
);

export { OverlordFeedProvider };
