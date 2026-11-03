/* @layer renderer-app @kind component */
import { Flex } from '@drizztdourden08/tessera';
import type { PrListProps } from '../changes.type';
import { sortPrs } from '../model';
import { PrCard } from './PrCard';

const PrList = (props: PrListProps) => {
  const { boardKey, prs, onLog } = props;
  return (
    <Flex direction="column" gap="sm" role="list" aria-label="Pull requests">
      {sortPrs(prs).map((pr) => <PrCard key={pr.id} boardKey={boardKey} pr={pr} onLog={onLog} />)}
    </Flex>
  );
};

export { PrList };
