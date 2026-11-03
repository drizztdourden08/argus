/* @layer renderer-app @kind component */
import type { ComponentType } from 'react';
import type { PaneBodyProps, RecordBodyProps } from './pane.type';

const itemBody = (ForItem: ComponentType<PaneBodyProps>, Otherwise: ComponentType<RecordBodyProps>): ComponentType<RecordBodyProps> => {
  const Body = ({ data }: RecordBodyProps) => (data.item ? <ForItem data={data.item} /> : <Otherwise data={data} />);
  return Body;
};

export { itemBody };
