/* @layer renderer-app @kind component */
import { Inline, Select, Tag, Text } from '@drizztdourden08/tessera';
import { ALL_RULES, CALM_LOOK } from '../processes.constants';
import type { ProcessFiltersProps } from '../processes.type';

const ProcessFilters = (props: ProcessFiltersProps) => {
  const { filters, onChange, total, look, rules } = props;
  const options = [{ value: ALL_RULES, label: 'All rules' }, ...rules.map((rule) => ({ value: rule, label: rule }))];
  const choices = [{ value: 'all', label: 'All', count: total }, { value: 'look', label: 'Needs a look', count: look }] as const;
  return (
    <Inline gap="md" align="center" wrap>
      <Inline gap="sm" align="center" wrap role="radiogroup" aria-label="Show processes">
        {choices.map((choice) => (
          <Tag pill key={choice.value} {...CALM_LOOK} role="radio" selected={filters.status === choice.value} onSelect={() => onChange({ ...filters, status: choice.value })}>
            <Inline gap="xs" align="baseline">
              {choice.label}
              <Text variant="caption" numeric>{choice.count}</Text>
            </Inline>
          </Tag>
        ))}
      </Inline>
      {rules.length > 0 && (
        <Inline gap="sm" align="center">
          <Text variant="caption">Rule</Text>
          <Select
            aria-label="Filter by rule"
            size="sm"
            inline
            options={options}
            value={filters.rule || ALL_RULES}
            onChange={(rule) => onChange({ ...filters, rule: rule === ALL_RULES ? '' : rule })}
          />
        </Inline>
      )}
    </Inline>
  );
};

export { ProcessFilters };
