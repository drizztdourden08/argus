/* @layer renderer-app @kind component */
import { Card, Flex } from '@drizztdourden08/tessera';
import type { PrCardProps } from '../changes.type';
import { PrChecks } from './PrChecks';
import { PrHead } from './PrHead';
import { PrMeta } from './PrMeta';

const PrCard = (props: PrCardProps) => {
  const { boardKey, pr, onLog } = props;
  return (
    <Card data-pr={pr.id} role="group" aria-label={`Pull request ${pr.id}: ${pr.title}`} edge="border">
      <Flex direction="column" gap="sm">
        <PrHead pr={pr} />
        <PrMeta pr={pr} />
        <PrChecks boardKey={boardKey} pr={pr} onLog={onLog} />
      </Flex>
    </Card>
  );
};

export { PrCard };
