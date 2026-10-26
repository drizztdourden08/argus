/* @layer renderer-app @kind logic */
import { useContext } from 'react';
import { OpenThreadContext } from './open-thread-context';

const useOpenThread = (): ((thread: string) => void) => useContext(OpenThreadContext);

export { useOpenThread };
