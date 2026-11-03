/* @layer renderer-app @kind component */
import { Flex, Text } from '@drizztdourden08/tessera';
import { depColor, useDep } from '../../../../../data/deps';
import { DepRecordLink, DepTag } from '../../../../../shell';
import type { DependencyViewProps } from '../drawer-props.type';
import { DEP_HEADINGS } from '../kanban.constants';
import { depView } from '../model';
import { CancelDependency } from './CancelDependency';
import { DependencyFacts } from './DependencyFacts';
import { DependencyHistory } from './DependencyHistory';
import { DependencyLink } from './DependencyLink';

const DependencyView = (props: DependencyViewProps) => {
  const { link, now } = props;
  const { doc } = useDep(link.dep);
  const view = depView(link, doc);
  return (
    <Flex direction="column" gap="sm">
      <Flex gap="sm" align="center" wrap>
        <DepTag color={depColor(view.state)}>{view.state}</DepTag>
        <DepRecordLink id={view.dep} />
        <Text variant="caption" tone="muted">{DEP_HEADINGS[view.side]}</Text>
        <DependencyLink view={view} />
        {view.cancellable && <CancelDependency dep={view.dep} />}
      </Flex>
      <DependencyFacts view={view} now={now} />
      <DependencyHistory rows={view.history} now={now} />
    </Flex>
  );
};

export { DependencyView };
