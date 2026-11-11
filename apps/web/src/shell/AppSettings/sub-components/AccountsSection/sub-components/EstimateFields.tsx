/* @layer renderer-app @kind component */
import { Button, Inline, Text } from '@drizztdourden08/tessera';
import { ESTIMATE_FIELDS, RESET_K_LABEL } from '../AccountsSection.constants';
import type { DraftProps } from '../AccountsSection.type';
import { NumberField } from './NumberField';

const EstimateFields = (props: DraftProps) => {
  const { draft, onChange } = props;
  const defaults = draft.estimateDefaults;
  return (
    <Inline gap="md" align="end" wrap>
      {ESTIMATE_FIELDS.map((field) => (
        <NumberField key={field.id} label={field.label} step={0.01} value={defaults[field.id]} onChange={(value) => onChange({ estimateDefaults: { ...defaults, [field.id]: value } })} />
      ))}
      <Text tone="muted">{`Learned k ${draft.k}`}</Text>
      <Button size="sm" variant="secondary" disabled={draft.k === 1} onClick={() => onChange({ k: 1 })}>{RESET_K_LABEL}</Button>
    </Inline>
  );
};

export { EstimateFields };
