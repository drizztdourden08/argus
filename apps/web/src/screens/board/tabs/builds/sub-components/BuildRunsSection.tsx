/* @layer renderer-app @kind component */
import { Box, Flex, Text } from '@drizztdourden08/tessera';
import type { BuildRunsProps } from '../builds-props.type';
import { RUNNING_TITLE } from '../builds.constants';
import { BuildRunRow } from './BuildRunRow';

const BuildRunsSection = (props: BuildRunsProps) => {
  const { runs, boardKey, now } = props;
  if (runs.length === 0) return null;
  return (
    <Box as="section" aria-label={RUNNING_TITLE} className="builds-runs">
      <Text as="h3" variant="overline" className="builds-runs__title">{`${RUNNING_TITLE} \u00b7 ${runs.length}`}</Text>
      <Flex direction="column" gap="xs" role="list" aria-label={RUNNING_TITLE} className="builds-runs__list">
        {runs.map((run) => <BuildRunRow key={run.id} run={run} boardKey={boardKey} now={now} />)}
      </Flex>
    </Box>
  );
};

export { BuildRunsSection };
