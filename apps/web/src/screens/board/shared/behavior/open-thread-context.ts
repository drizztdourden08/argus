/* @layer renderer-app @kind logic */
import { createContext } from 'react';

const nobody = (): void => undefined;

const OpenThreadContext = createContext<(thread: string) => void>(nobody);

export { OpenThreadContext };
