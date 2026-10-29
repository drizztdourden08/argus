/* @layer renderer-app @kind component */
import { Callout, Flex, Text } from '@drizztdourden08/tessera';
import { usePrChecks } from '../behavior/usePrChecks';
import { CHECKS_NONE, CHECKS_READING } from '../changes.constants';
import type { PrChecksProps } from '../changes.type';
import { sortRuns } from '../model';
import { CheckRunView } from './CheckRunView';

const PrChecks = (props: PrChecksProps) => {
  const { boardKey, pr, onLog } = props;
  const { data, error } = usePrChecks(boardKey, pr);
  if (error && !data) return <Callout tone="danger">{`The check runs could not be read: ${error}.`}</Callout>;
  if (!data) return <Text variant="caption" tone="muted">{CHECKS_READING}</Text>;
  if (data.length === 0) return <Text variant="caption" tone="muted">{CHECKS_NONE}</Text>;
  return (
    <Flex direction="column" gap="xs" role="group" aria-label={`Check runs of ${pr.id}`}>
      {sortRuns(data).map((run, index) => <CheckRunView key={run.id} run={run} first={index === 0} onLog={onLog} />)}
    </Flex>
  );
};

export { PrChecks };
