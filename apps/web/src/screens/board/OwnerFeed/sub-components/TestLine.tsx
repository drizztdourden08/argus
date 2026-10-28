/* @layer renderer-app @kind component */
import { Button, Flex, Tag, Text } from '@drizztdourden08/tessera';
import { smartTimes } from '../../../../format';
import { Age } from '../../../../shell';
import { useNavigate } from 'react-router';
import { testLine } from '../behavior/test-line';
import type { TestLineProps } from '../OwnerFeed.type';
import './TestLine.css';

const TestLine = (props: TestLineProps) => {
  const { card, handed } = props;
  const navigate = useNavigate();
  const line = testLine(card);
  return (
    <Flex align="center" gap="sm" wrap>
      <Flex className="test-line__main" align="center" gap="sm" wrap>
        {line.build ? <Tag>{line.build === handed ? `${line.build} · handed to you` : line.build}</Tag> : null}
        <Text className="test-line__text" title={line.hover}>
          {line.count ? <Text weight={600}>{line.count}</Text> : null}
          {line.count ? ' · ' : null}
          {smartTimes(line.title)}
        </Text>
      </Flex>
      <Text variant="caption" tone="muted">{line.thread}</Text>
      <Text variant="caption" tone="muted"><Age value={card.at} /></Text>
      <Button size="sm" variant="tertiary" onClick={() => void navigate({ hash: line.open })}>Open</Button>
    </Flex>
  );
};

export { TestLine };
