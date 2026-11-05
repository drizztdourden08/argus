/* @layer renderer-app @kind component */
import { Card, Disclosure, Flex, Text } from '@drizztdourden08/tessera';
import { useSentDown } from '../behavior/useSentDown';
import { NOTHING_SENT } from '../boards-send.constants';
import { SentRow } from './SentRow';

const SentDown = () => {
  const { total, rows } = useSentDown();
  return (
    <Card>
      <Disclosure summary={`Sent down (${total})`}>
        {rows.length === 0
          ? <Text tone="muted">{NOTHING_SENT}</Text>
          : (
            <Flex direction="column" gap="xs" role="list" aria-label="Sent down">
              {rows.map((entry) => <SentRow key={entry.id} entry={entry} />)}
            </Flex>
          )}
      </Disclosure>
    </Card>
  );
};

export { SentDown };
