/* @layer renderer-app @kind component */
import './BundleCard.css';
import { Box, Button, DateText, Flex, Icon, Spacer, Tag, Text } from '@drizztdourden08/tessera';
import { countsText } from '../model';
import { BUNDLE_TAGS } from '../tests.constants';
import type { ClosedBundleCardProps } from '../tests-props.type';
import { ClosedTestRow } from './ClosedTestRow';

const ClosedBundleCard = (props: ClosedBundleCardProps) => {
  const { group, open, onToggle } = props;
  const { build } = group;
  return (
    <Box className="bundle bundle--closed" data-bundle={group.tag}>
      <Flex gap="sm" align="center" wrap className="bundle__head">
        <Button
          size="sm"
          variant="ghost"
          className="bundle__toggle"
          icon={<Icon name={open ? 'chevron-down' : 'chevron-right'} />}
          aria-expanded={open}
          title={`${open ? 'Fold' : 'Open'} ${group.tag}`}
          onClick={() => onToggle(!open)}
        >
          <Text mono tone="muted" weight={600} className="bundle__tag">{group.tag}</Text>
          {build.title && <Text tone="muted" className="bundle__title">{build.title}</Text>}
        </Button>
        <Spacer />
        <Tag {...BUNDLE_TAGS.closed} title={build.closeNote ?? 'This bundle was closed: finished and landed.'}>
          {build.closedAt ? <>closed <DateText value={build.closedAt} format="elapsed" length="short" /> ago{build.closedBy ? ` by ${build.closedBy}` : ''}</> : 'closed'}
        </Tag>
        <Text variant="caption" tone="muted">{countsText(group.counts)}</Text>
      </Flex>
      <Box className="bundle__body" hidden={!open}>
        {group.tests.length > 0
          ? group.tests.map((test) => <ClosedTestRow key={test.id} test={test} />)
          : <Text tone="muted" className="test-section__empty">No test in this bundle.</Text>}
      </Box>
    </Box>
  );
};

export { ClosedBundleCard };
