/* @layer renderer-app @kind component */
import { Inline, Select } from '@drizztdourden08/tessera';
import { ARCHIVE_CHOICES, THRESHOLD_FIELDS } from '../AccountsSection.constants';
import type { DraftProps } from '../AccountsSection.type';
import { NumberField } from './NumberField';

const ThresholdFields = (props: DraftProps) => {
  const { draft, onChange } = props;
  return (
    <Inline gap="md" align="end" wrap>
      {THRESHOLD_FIELDS.map((field) => (
        <NumberField key={field.id} label={field.label} value={draft[field.id]} onChange={(value) => onChange({ [field.id]: value })} />
      ))}
      <Select size="sm" width="fit" aria-label="Archive rule" options={ARCHIVE_CHOICES} value={draft.archiveRule} onChange={(archiveRule) => onChange({ archiveRule: archiveRule as typeof draft.archiveRule })} />
    </Inline>
  );
};

export { ThresholdFields };
