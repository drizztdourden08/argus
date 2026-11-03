/* @layer renderer-app @kind component */
import { Flex, Pressable, StatRow, Text } from '@drizztdourden08/tessera';
import { flaggedResults } from './model/flagged-results';
import { summarize } from './model/summarize';
import { RunStart } from './RunStart';
import { SUMMARY_ROWS } from './run.constants';
import type { RunSummaryProps } from './run.type';

const RunSummary = (props: RunSummaryProps) => {
  const { data, run, ids, ops } = props;
  const summary = summarize(run, ids);
  const flagged = flaggedResults(run, ids);
  return (
    <Flex direction="column" gap="md" role="group" aria-label={`Summary of run ${run.n}`}>
      <Text weight={600}>{`Run ${run.n} ended: ${summary.total} cases`}</Text>
      <Flex gap="lg" wrap>
        {SUMMARY_ROWS.map(([key, label]) => (
          <StatRow key={key} label={label} value={String(summary[key])} />
        ))}
      </Flex>
      {flagged.length > 0 && (
        <Flex direction="column" gap="xs">
          <Text variant="label">Needs a look</Text>
          {flagged.map((result) => (
            <Flex key={result.test} gap="sm" align="baseline" wrap>
              <Pressable aria-label={`Open ${result.test}`} onClick={() => data.open(result.test)}>
                <Text as="span" variant="caption" weight={600} mono>{result.test}</Text>
              </Pressable>
              <Text as="span" variant="caption" tone="muted">{result.status}</Text>
              <Text as="span" smartTimes>{result.note}</Text>
            </Flex>
          ))}
        </Flex>
      )}
      <RunStart bundle={run.bundle} ops={ops} again />
    </Flex>
  );
};

export { RunSummary };
