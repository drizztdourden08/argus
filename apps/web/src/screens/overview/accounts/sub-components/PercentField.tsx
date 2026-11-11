/* @layer renderer-app @kind component */
import { NumberInput } from '@drizztdourden08/tessera';
import { useEffect, useState } from 'react';
import type { PercentFieldProps } from '../accounts.type';

const PercentField = (props: PercentFieldProps) => {
  const { value, label, disabled, onCommit } = props;
  const [draft, setDraft] = useState(value);
  useEffect(() => setDraft(value), [value]);
  const commit = () => draft !== value && Number.isFinite(draft) && onCommit(Math.min(100, Math.max(1, draft)));
  return <NumberInput size="sm" aria-label={label} min={1} max={100} value={draft} disabled={disabled} onChange={setDraft} onBlur={commit} />;
};

export { PercentField };
