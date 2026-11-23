/* @layer renderer-app @kind component */
import { Flex } from '@drizztdourden08/tessera';
import type { PrListProps } from '../changes.type';
import { PrCard } from './PrCard';

const PrList = (props: PrListProps) => {
  const { boardKey, rows, threads, compact } = props;
  return (
    <Flex direction="column" gap="sm" role="list" aria-label="Pull requests">
      {rows.map((pr) => <PrCard key={pr.id} boardKey={boardKey} pr={pr} threads={threads} compact={compact} />)}
    </Flex>
  );
};

export { PrList };
