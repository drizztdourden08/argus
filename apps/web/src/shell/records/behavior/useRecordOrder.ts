/* @layer renderer-app @kind logic */
import { useContext, useEffect } from 'react';
import { RecordOrderContext } from './record-order-context';

const useRecordOrder = (ids: readonly string[]): void => {
  const { set } = useContext(RecordOrderContext);
  const key = ids.join('|');
  useEffect(() => {
    set(key === '' ? [] : key.split('|'));
  }, [key, set]);
};

export { useRecordOrder };
