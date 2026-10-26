/* @layer renderer-app @kind logic */
import { useContext } from 'react';
import type { ScopeStreams } from '../../stream';
import { DataContext } from '../data-context';

const useStreams = (): ScopeStreams => {
  const streams = useContext(DataContext);
  if (!streams) throw new Error('DataProvider is missing above this component');
  return streams;
};

export { useStreams };
