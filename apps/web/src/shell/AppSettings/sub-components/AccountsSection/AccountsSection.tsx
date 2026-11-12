/* @layer renderer-app @kind component */
import { Button, Callout, Inline, Paragraph, SettingsSection, Stack, toast, Toggle } from '@drizztdourden08/tessera';
import { useAccountsSettings } from '../../../../data/accounts';
import { PORTFOLIO_SCOPE, useCollection } from '../../../../data/react';
import type { AccountDoc } from '../../../../data/types';
import { ACCOUNTS_COLL, ACCOUNTS_NOTE, ACCOUNTS_TITLE, ADD_LABEL, FORCE_QUIT_LABEL, SAVED_TEXT, SAVE_LABEL } from './AccountsSection.constants';
import { useAccountsDraft } from './behavior/useAccountsDraft';
import { AccountRow } from './sub-components/AccountRow';
import { EstimateFields } from './sub-components/EstimateFields';
import { ThresholdFields } from './sub-components/ThresholdFields';

const AccountsSection = () => {
  const api = useAccountsSettings();
  const docs = useCollection<AccountDoc>(PORTFOLIO_SCOPE, ACCOUNTS_COLL).docs;
  const { draft, patch, changeAccount, addAccount } = useAccountsDraft(api.settings);
  if (api.problem && !draft) return <Callout tone="danger">{`The accounts settings could not be read: ${api.problem}.`}</Callout>;
  if (!draft) return null;
  const saved = new Set(api.settings?.accounts.map((item) => item.key));
  const save = () => void api.save(draft).then((ok) => ok && toast({ variant: 'success', message: SAVED_TEXT }));
  return (
    <SettingsSection id="accounts" title={ACCOUNTS_TITLE}>
      <Stack gap="md">
        <Paragraph tone="muted">{ACCOUNTS_NOTE}</Paragraph>
        {api.problem && <Callout tone="danger">{api.problem}</Callout>}
        {draft.accounts.map((item, index) => (
          <AccountRow key={item.key || `new-${index}`} item={item} isNew={!saved.has(item.key)} doc={docs.find((doc) => doc.id === item.key)} onChange={(change) => changeAccount(index, change)} />
        ))}
        <Inline gap="sm">
          <Button size="sm" variant="secondary" onClick={addAccount}>{ADD_LABEL}</Button>
          <Button size="sm" disabled={api.busy} onClick={save}>{SAVE_LABEL}</Button>
        </Inline>
        <Toggle checked={draft.forceQuit === true} onChange={(forceQuit) => patch({ forceQuit })} label={FORCE_QUIT_LABEL} />
        <ThresholdFields draft={draft} onChange={patch} />
        <EstimateFields draft={draft} onChange={patch} />
      </Stack>
    </SettingsSection>
  );
};

export { AccountsSection };
