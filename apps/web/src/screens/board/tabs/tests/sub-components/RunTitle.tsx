/* @layer renderer-app @kind component */
import { Flex, Text } from '@drizztdourden08/tessera';
import { ShownCode, TypeIcon } from '../../../../../shell';
import { RUN_KIND } from '../tests.constants';
import type { RunTitleProps } from '../tests-head-props.type';
import { RunChips } from './RunChips';

const RunTitle = (props: RunTitleProps) => {
  const { view, muted = false } = props;
  const title = view.item.title || view.plan?.title;
  return (
    <Flex gap="xs" align="center" wrap className="run-card__name">
      <TypeIcon kind={RUN_KIND} />
      <Text font="title" tone={muted ? 'muted' : undefined} className="run-card__tag"><ShownCode id={view.item.id} /></Text>
      {title && <Text smartTimes tone={muted ? 'muted' : 'dim'} className="run-card__title">{title}</Text>}
      <RunChips view={view} />
    </Flex>
  );
};

export { RunTitle };
