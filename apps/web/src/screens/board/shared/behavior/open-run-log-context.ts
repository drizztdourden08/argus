/* @layer renderer-app @kind logic */
import { createContext } from 'react';

const nobody = (): void => undefined;

const OpenRunLogContext = createContext<(run: string) => void>(nobody);

export { OpenRunLogContext };
