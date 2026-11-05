/* @layer renderer-app @kind component */
import { TextInput } from '@drizztdourden08/tessera';
import { useState } from 'react';
import type { ProfileTextProps } from '../AppSettings.type';

const ProfileText = (props: ProfileTextProps) => {
  const { value, label, maxLength, disabled, onSave } = props;
  const [draft, setDraft] = useState(value);
  const commit = (next: string) => {
    if (next.trim() !== value) onSave(next);
  };
  return (
    <TextInput
      size="sm"
      aria-label={label}
      value={draft}
      maxLength={maxLength}
      disabled={disabled}
      onChange={(event) => setDraft(event.target.value)}
      onBlur={() => commit(draft)}
      onEnter={commit}
    />
  );
};

export { ProfileText };
