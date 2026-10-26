/* @layer renderer-app @kind component */
import { Disclosure, ListItemList, Text } from '@drizztdourden08/tessera';
import { useNow } from '../../clock';
import { useSentDown } from '../behavior/useSentDown';
import { NOTHING_SENT } from '../dashboards.constants';
import { SentRow } from './SentRow';

const SentDown = () => {
  const { total, rows } = useSentDown();
  const now = useNow();
  return (
    <Disclosure summary={`Sent down (${total})`}>
      {rows.length === 0
        ? <Text tone="muted">{NOTHING_SENT}</Text>
        : (
          <ListItemList label="Sent down">
            {rows.map((entry) => <SentRow key={entry.id} entry={entry} now={now} />)}
          </ListItemList>
        )}
    </Disclosure>
  );
};

export { SentDown };
