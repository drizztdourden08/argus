/* @layer renderer-app @kind logic */
import { createContext } from 'react';

const nobody = (): void => undefined;

const OpenEntryContext = createContext<(entry: string) => void>(nobody);

export { OpenEntryContext };
