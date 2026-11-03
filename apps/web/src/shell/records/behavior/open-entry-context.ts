/* @layer renderer-app @kind logic */
import { createContext } from 'react';

const nobody = (): void => undefined;

const OpenEntryContext = createContext<(entry: string | null) => void>(nobody);

export { OpenEntryContext };
