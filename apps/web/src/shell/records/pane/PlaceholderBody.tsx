/* @layer renderer-app @kind component */
import type { ComponentType } from 'react';
import type { RecordBodyProps } from './pane.type';
import { TabPlaceholder } from './TabPlaceholder';

const placeholderBody = (tab: string, label: string): ComponentType<RecordBodyProps> => {
  const Body = ({ data }: RecordBodyProps) => <TabPlaceholder tab={tab} label={label} data={data} />;
  return Body;
};

export { placeholderBody };
