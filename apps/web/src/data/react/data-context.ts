/* @layer renderer-app @kind logic */
import { createContext } from 'react';
import type { ScopeStreams } from '../stream';

const DataContext = createContext<ScopeStreams | null>(null);

export { DataContext };
