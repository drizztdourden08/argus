/* @layer renderer-app @kind component */
import { useMemo, useState } from 'react';
import { RecordOrderContext } from '../behavior/record-order-context';
import type { RecordPaneScopeProps } from '../records.type';

const RecordPaneScope = (props: RecordPaneScopeProps) => {
  const [ids, set] = useState<readonly string[]>([]);
  const value = useMemo(() => ({ ids, set }), [ids]);
  return <RecordOrderContext.Provider value={value}>{props.children}</RecordOrderContext.Provider>;
};

export { RecordPaneScope };
