/* @layer renderer-app @kind component */
import './RunCard.css';
import { Card, Disclosure, Flex, Tag, Text } from '@drizztdourden08/tessera';
import { countsText } from '../model';
import { RUN_TAGS, RUN_TIPS } from '../tests.constants';
import type { ClosedRunCardProps } from '../tests-props.type';
import { ClosedTestRow } from './ClosedTestRow';
import { RunName } from './RunName';

const ClosedRunCard = (props: ClosedRunCardProps) => {
  const { view, open, onToggle } = props;
  return (
    <Card flush className="run-card run-card--closed" data-swap-item data-run={view.item.id}>
      <Disclosure
        open={open}
        onOpenChange={onToggle}
        summary={<RunName id={view.item.id} title={view.plan?.title} muted />}
        end={(
          <Flex gap="sm" align="center" className="run-card__end">
            <Tag {...(RUN_TAGS[view.stage] ?? RUN_TAGS.passed)} title={RUN_TIPS[view.stage]}>{view.stage}</Tag>
            <Text variant="caption" tone="muted">{countsText(view.counts)}</Text>
          </Flex>
        )}
      >
        {view.tests.length > 0
          ? view.tests.map((test) => <ClosedTestRow key={test.id} test={test} />)
          : <Text tone="muted" className="test-section__empty">No test in this run.</Text>}
      </Disclosure>
    </Card>
  );
};

export { ClosedRunCard };
