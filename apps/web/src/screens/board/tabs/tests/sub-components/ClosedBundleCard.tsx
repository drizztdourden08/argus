/* @layer renderer-app @kind component */
import './BundleCard.css';
import { Card, Disclosure, Flex, Tag, Text } from '@drizztdourden08/tessera';
import { Age } from '../../../../../shell';
import { countsText } from '../model';
import { BUNDLE_TAGS } from '../tests.constants';
import type { ClosedBundleCardProps } from '../tests-props.type';
import { BundleName } from './BundleName';
import { ClosedTestRow } from './ClosedTestRow';

const ClosedBundleCard = (props: ClosedBundleCardProps) => {
  const { group, open, onToggle } = props;
  const { build } = group;
  return (
    <Card flush className="bundle bundle--closed" data-bundle={group.tag}>
      <Disclosure
        open={open}
        onOpenChange={onToggle}
        summary={<BundleName tag={group.tag} title={build.title} muted />}
        end={(
          <Flex gap="sm" align="center" className="bundle__end">
            <Tag {...BUNDLE_TAGS.closed} title={build.closeNote ?? 'This bundle was closed: finished and landed.'}>
              {build.closedAt ? <>closed <Age value={build.closedAt} /> ago{build.closedBy ? ` by ${build.closedBy}` : ''}</> : 'closed'}
            </Tag>
            <Text variant="caption" tone="muted">{countsText(group.counts)}</Text>
          </Flex>
        )}
      >
        {group.tests.length > 0
          ? group.tests.map((test) => <ClosedTestRow key={test.id} test={test} />)
          : <Text tone="muted" className="test-section__empty">No test in this bundle.</Text>}
      </Disclosure>
    </Card>
  );
};

export { ClosedBundleCard };
