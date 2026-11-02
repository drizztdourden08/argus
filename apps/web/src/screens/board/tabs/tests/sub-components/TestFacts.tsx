/* @layer renderer-app @kind component */
import { Flex, InlineEdit, Tag, Text } from '@drizztdourden08/tessera';
import { FactSelect } from '../../../shared';
import { useTestsShared } from '../behavior/useTestsShared';
import { fieldValidator, itemChoices } from '../model';
import { MODE_OPTIONS } from '../tests.constants';
import type { TestFactsProps } from '../tests-drawer-props.type';
import { FactTags } from './FactTags';

const validateCovers = fieldValidator('covers');
const validateBuild = fieldValidator('build');

const TestFacts = (props: TestFactsProps) => {
  const { test, handed } = props;
  const { ops, items } = useTestsShared();
  return (
    <Flex direction="column" gap="sm">
      <Flex gap="sm" align="center" wrap>
        <FactTags {...props} />
      </Flex>
      <Flex gap="md" align="center" wrap>
        <FactSelect label="Mode" options={MODE_OPTIONS} value={test.mode} onPick={(mode) => mode !== test.mode && void ops.edit(test, { mode })} />
        <FactSelect label="Item" options={itemChoices(items)} value={test.item ?? ''} onPick={(item) => item !== test.item && void ops.move(test, item)} />
        <Flex gap="xs" align="center">
          <Text variant="caption" tone="muted">Covers</Text>
          <InlineEdit label="covers" placeholder="add" value={test.covers ?? ''} validate={validateCovers} onCommit={(covers) => void ops.edit(test, { covers: covers.trim() })} />
        </Flex>
        <Flex gap="xs" align="center">
          <Text variant="caption" tone="muted">Build</Text>
          <InlineEdit label="build" placeholder="add" value={test.build ?? ''} validate={validateBuild} onCommit={(build) => void ops.edit(test, { build: build.trim() })} />
          {handed && <Tag color="neutral" fill="solid">handed to you</Tag>}
        </Flex>
      </Flex>
    </Flex>
  );
};

export { TestFacts };
