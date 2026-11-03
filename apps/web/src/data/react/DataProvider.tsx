/* @layer renderer-app @kind component */
import { useMemo } from 'react';
import { connectHub } from '../hub/connect-hub';
import { TabLink } from '../hub/tab-link';
import { createScopeStreams } from '../stream';
import { DataContext } from './data-context';
import type { DataProviderProps } from './react.type';

const DataProvider = (props: DataProviderProps) => {
  const { streams, children } = props;
  const value = useMemo(() => streams ?? createScopeStreams(new TabLink(connectHub())), [streams]);
  return <DataContext value={value}>{children}</DataContext>;
};

export { DataProvider };
