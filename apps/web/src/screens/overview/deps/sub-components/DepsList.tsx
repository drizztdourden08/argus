/* @layer renderer-app @kind component */
import { OverviewList } from '../../records';
import type { DepsListProps } from '../deps.type';
import { DepRow } from './DepRow';

const DepsList = (props: DepsListProps) => {
  const { docs, all, now } = props;
  return (
    <OverviewList label="Dependencies between projects">
      {docs.map((doc) => <DepRow key={doc.id} doc={doc} all={all} now={now} />)}
    </OverviewList>
  );
};

export { DepsList };
