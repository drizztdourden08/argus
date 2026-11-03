/* @layer renderer-app @kind logic */
import { useContext } from 'react';
import { OpenRunLogContext } from './open-run-log-context';

const useOpenRunLog = (): ((run: string) => void) => useContext(OpenRunLogContext);

export { useOpenRunLog };
