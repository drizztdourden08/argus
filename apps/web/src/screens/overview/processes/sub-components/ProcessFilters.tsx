/* @layer renderer-app @kind component */
import { Inline, SegmentedControl, Select } from '@drizztdourden08/tessera';
import { ALL_RULES } from '../processes.constants';
import type { ProcessFiltersProps } from '../processes.type';

const ProcessFilters = (props: ProcessFiltersProps) => {
  const { filters, onChange, total, look, rules } = props;
  const options = [{ value: ALL_RULES, label: 'All rules' }, ...rules.map((rule) => ({ value: rule, label: rule }))];
  return (
    <Inline gap="md" align="center" wrap>
      <SegmentedControl
        aria-label="Show processes"
        value={filters.status}
        options={[
          { value: 'all', label: `All ${total}` },
          { value: 'look', label: `Needs a look ${look}` },
        ]}
        onChange={(status) => onChange({ ...filters, status })}
      />
      {rules.length > 0 && (
        <Select
          aria-label="Filter by rule"
          options={options}
          value={filters.rule || ALL_RULES}
          onChange={(rule) => onChange({ ...filters, rule: rule === ALL_RULES ? '' : rule })}
        />
      )}
    </Inline>
  );
};

export { ProcessFilters };
