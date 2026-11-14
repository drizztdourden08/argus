/* @layer renderer-app @kind component */
import { Button, Inline, PasswordInput, SettingsSection } from '@drizztdourden08/tessera';
import { useState } from 'react';
import { useOverlordKey } from '../../../../data/overlord';
import { keyLine } from './behavior/key-line';
import { KEY_FIELD_LABEL, KEY_PLACEHOLDER, KEY_ROW_TITLE, KEY_TITLE, REMOVE_LABEL, SAVE_LABEL } from './OverlordKeySection.constants';

const OverlordKeySection = () => {
  const api = useOverlordKey();
  const [draft, setDraft] = useState('');
  const save = () => void api.save(draft).then((ok) => ok && setDraft(''));
  const control = (
    <Inline gap="sm">
      <PasswordInput size="sm" aria-label={KEY_FIELD_LABEL} placeholder={KEY_PLACEHOLDER} value={draft} onChange={setDraft} />
      <Button size="sm" disabled={api.busy || draft.trim() === ''} onClick={save}>{SAVE_LABEL}</Button>
      <Button size="sm" variant="secondary" disabled={api.busy || !api.status?.set} onClick={() => void api.remove()}>{REMOVE_LABEL}</Button>
    </Inline>
  );
  return (
    <SettingsSection
      id="overlord-key"
      title={KEY_TITLE}
      rows={[{ id: 'key', title: KEY_ROW_TITLE, description: keyLine(api.status), problem: api.problem ?? undefined, input: { kind: 'custom', control } }]}
    />
  );
};

export { OverlordKeySection };
