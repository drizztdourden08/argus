/* @layer renderer-app @kind component */
import { Flex, Text } from '@drizztdourden08/tessera';
import { countsText } from '../model';
import type { RunHeadProps } from '../tests-props.type';
import { HandButton } from './HandButton';
import { RunProgress } from './RunProgress';
import { RunTags } from './RunTags';

const RunHead = (props: RunHeadProps) => {
  const { view } = props;
  return (
    <Flex gap="sm" align="center" className="run-card__end">
      <RunTags view={view} />
      <Text variant="caption" tone="dim">{countsText(view.counts)}</Text>
      <RunProgress counts={view.counts} />
      {(view.stage === 'ready' || view.handed) && <HandButton run={view.item} handed={view.handed} />}
    </Flex>
  );
};

export { RunHead };
