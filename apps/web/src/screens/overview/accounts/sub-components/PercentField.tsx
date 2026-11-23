/* @layer renderer-app @kind component */
import { NumberInput } from '@drizztdourden08/tessera';
import { useEffect, useState } from 'react';
import { MAX_PERCENT, MIN_PERCENT } from '../accounts.constants';
import type { PercentFieldProps } from '../accounts.type';
import { clampPercent } from '../model';
import { PercentMark } from './PercentMark';

const PercentField = (props: PercentFieldProps) => {
  const { value, label, disabled, onCommit } = props;
  const [draft, setDraft] = useState(value);
  useEffect(() => setDraft(value), [value]);
  const commit = (next: number) => {
    const percent = clampPercent(next, value);
    setDraft(percent);
    if (percent !== value) onCommit(percent);
  };
  const step = (dir: 1 | -1) => commit((Number.isFinite(draft) ? draft : value) + dir);
  return (
    <NumberInput
      size="sm"
      sizeToContent
      end={{ icon: <PercentMark /> }}
      aria-label={label}
      min={MIN_PERCENT}
      max={MAX_PERCENT}
      value={draft}
      disabled={disabled}
      stepDisabled={[draft <= MIN_PERCENT, draft >= MAX_PERCENT]}
      onChange={setDraft}
      onStep={step}
      onBlur={() => commit(draft)}
      onKeyDown={(event) => event.key === 'Enter' && commit(draft)}
    />
  );
};

export { PercentField };
