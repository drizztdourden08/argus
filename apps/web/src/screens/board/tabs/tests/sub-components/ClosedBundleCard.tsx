/* @layer renderer-app @kind component */
import { Card, DateText, Disclosure, Flex, List, Status, Text } from '@drizztdourden08/tessera';
import { countsText } from '../model';
import { BUNDLE_STATUSES } from '../tests.constants';
import type { ClosedBundleCardProps } from '../tests-props.type';
import { ClosedTestRow } from './ClosedTestRow';

const ClosedBundleCard = (props: ClosedBundleCardProps) => {
  const { group, open, onToggle } = props;
  const { build } = group;
  const head = (
    <Flex gap="sm" align="center" wrap>
      <Text mono tone="muted">{group.tag}</Text>
      {build.title && <Text tone="muted">{build.title}</Text>}
      <Status map={BUNDLE_STATUSES} value="closed" variant="pill" title={build.closeNote ?? 'This bundle was closed: finished and landed.'} />
      {build.closedAt && (
        <Text variant="caption" tone="muted">closed <DateText value={build.closedAt} format="relative" />{build.closedBy ? ` by ${build.closedBy}` : ''}</Text>
      )}
      <Text variant="caption" tone="muted">{countsText(group.counts)}</Text>
    </Flex>
  );
  return (
    <Card data-bundle={group.tag}>
      <Disclosure open={open} onOpenChange={onToggle} summary={head}>
        {group.tests.length > 0 ? (
          <List>{group.tests.map((test) => <ClosedTestRow key={test.id} test={test} />)}</List>
        ) : (
          <Text tone="muted">No test in this bundle.</Text>
        )}
      </Disclosure>
    </Card>
  );
};

export { ClosedBundleCard };
