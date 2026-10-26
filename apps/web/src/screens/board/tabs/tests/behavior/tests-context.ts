/* @layer renderer-app @kind logic */
import { createContext } from 'react';
import type { TestsShared } from '../tests.type';

const TestsContext = createContext<TestsShared | null>(null);

export { TestsContext };
