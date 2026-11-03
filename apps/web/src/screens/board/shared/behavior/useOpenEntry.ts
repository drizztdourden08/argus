/* @layer renderer-app @kind logic */
import { useContext } from 'react';
import { OpenEntryContext } from './open-entry-context';

const useOpenEntry = (): ((entry: string) => void) => useContext(OpenEntryContext);

export { useOpenEntry };
