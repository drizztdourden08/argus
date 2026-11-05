/* @layer renderer-app @kind component */
import { Checkbox, Select } from '@drizztdourden08/tessera';
import { ANY_LABEL, ANY_VALUE } from '../BoardTabsRow.constants';
import type { TabFieldControlProps } from '../BoardTabsRow.type';

const TabFieldControl = <F,>({ field, filters, patch }: TabFieldControlProps<F>) => {
  if (field.kind === 'flag') {
    return <Checkbox boxed size="sm" label={field.label} checked={field.read(filters)} onChange={(on) => patch(field.write(on))} />;
  }
  if (field.kind === 'multi') {
    return (
      <Select
        size="sm"
        width="fit"
        aria-label={field.label}
        prefix={`${field.label}:`}
        multiDisplay="count"
        options={field.options}
        values={field.read(filters)}
        onValuesChange={(values) => patch(field.write(values))}
      />
    );
  }
  return (
    <Select
      size="sm"
      width="fit"
      aria-label={field.label}
      prefix={`${field.label}:`}
      options={[{ value: ANY_VALUE, label: ANY_LABEL }, ...field.options]}
      value={field.read(filters) || ANY_VALUE}
      onChange={(value) => patch(field.write(value === ANY_VALUE ? '' : value))}
    />
  );
};

export { TabFieldControl };
