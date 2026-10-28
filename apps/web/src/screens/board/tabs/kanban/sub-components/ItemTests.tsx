/* @layer renderer-app @kind component */
import { Card, Flex, Pressable, Tag, Text } from '@drizztdourden08/tessera';
import { useNavigate } from 'react-router';
import { smartTimes } from '../../../../../format';
import { useDoc } from '../../../../../data/react';
import type { BoardMeta } from '../../../../../data/types';
import type { ItemTestsProps } from '../drawer-props.type';
import { DrawerSection } from './DrawerSection';
import { EmptyLine } from './EmptyLine';
import { StatusTag } from './StatusTag';

const ItemTests = (props: ItemTestsProps) => {
  const { boardKey, tests } = props;
  const handed = useDoc<BoardMeta>(boardKey, 'meta', 'board').doc?.handedBuild ?? null;
  const navigate = useNavigate();
  const passing = tests.filter((test) => test.status === 'pass').length;
  return (
    <DrawerSection title={`Tests · ${passing}/${tests.length} passing`}>
      {tests.length === 0 && <EmptyLine>No test documented for this item.</EmptyLine>}
      <Flex direction="column" gap="xs">
        {tests.map((test) => (
          <Card key={test.id} variant="interactive">
            <Pressable aria-label={`Open ${test.id}: ${test.title}`} onClick={() => void navigate({ hash: test.id })}>
              <Flex direction="column" gap="xs">
                <Flex gap="xs" align="center" wrap>
                  <Text variant="caption" weight={600} mono>{test.id}</Text>
                  <StatusTag status={test.status} />
                  <StatusTag status={test.mode} />
                  {test.build ? <Tag color="neutral" title={test.build === handed ? `Build ${test.build} is the one handed to you` : `Build ${test.build}`}>{test.build === handed ? `${test.build} · handed to you` : test.build}</Tag> : null}
                </Flex>
                <Text>{smartTimes(test.title)}</Text>
              </Flex>
            </Pressable>
          </Card>
        ))}
      </Flex>
    </DrawerSection>
  );
};

export { ItemTests };
