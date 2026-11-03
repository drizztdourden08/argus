/* @layer renderer-app @kind component */
import { Grid, Icon, Inline, Tag, Text } from '@drizztdourden08/tessera';
import { BAD_LOOK, CALM_LOOK } from '../../../../shell';
import type { CounterTileProps } from '../dashboards.type';

const toneOf = (value: number, bad: boolean): 'danger' | 'muted' | undefined => {
  if (bad) return 'danger';
  return value === 0 ? 'muted' : undefined;
};

const CounterTile = (props: CounterTileProps) => {
  const { icon, value, label, bad = false } = props;
  return (
    <Grid.Cell>
      <Tag {...(bad ? BAD_LOOK : CALM_LOOK)} title={`${value} ${label}`}>
        <Inline gap="2xs" align="center">
          <Text tone={value > 0 && !bad ? 'success' : undefined}><Icon name={icon} size={12} /></Text>
          <Text numeric weight={value > 0 ? 600 : undefined} tone={toneOf(value, bad)}>{value}</Text>
        </Inline>
      </Tag>
    </Grid.Cell>
  );
};

export { CounterTile };
