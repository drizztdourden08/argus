/* @layer renderer-app @kind logic */
import { createContext } from 'react';
import type { OverlordFeedApi } from '../OverlordChat.type';

const OverlordFeedContext = createContext<OverlordFeedApi | null>(null);

export { OverlordFeedContext };
