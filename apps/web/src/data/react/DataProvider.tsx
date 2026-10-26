/* @layer renderer-app @kind component */
import { useMemo } from 'react';
import { createScopeStreams } from '../stream';
import { DataContext } from './data-context';
import type { DataProviderProps } from './react.type';

const DataProvider = (props: DataProviderProps) => {
  const { streams, children } = props;
  const own = useMemo(() => createScopeStreams(), []);
  return <DataContext value={streams ?? own}>{children}</DataContext>;
};

export { DataProvider };
