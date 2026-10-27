/* @layer renderer-app @kind component */
import { Grid } from '@drizztdourden08/tessera';
import type { CardCountersProps } from '../dashboards.type';
import { cardCounts } from '../model';
import { CounterTile } from './CounterTile';

const CardCounters = (props: CardCountersProps) => {
  const { agents, items, notPassing, failing } = cardCounts(props.card);
  return (
    <Grid columns={3} gap="xs">
      <CounterTile icon="cpu" value={agents} label="agents working" />
      <CounterTile icon="columns-3" value={items} label="Kanban items outstanding" />
      <CounterTile icon="list-checks" value={notPassing} label="tests not passing" bad={failing > 0} />
    </Grid>
  );
};

export { CardCounters };
