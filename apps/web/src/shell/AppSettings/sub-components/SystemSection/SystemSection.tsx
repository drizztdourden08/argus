/* @layer renderer-app @kind component */
import { FactsPanel, Paragraph, SettingsSection } from '@drizztdourden08/tessera';
import { systemFacts } from './behavior/system-facts';
import { useSystemReads } from './behavior/useSystemReads';

const SystemSection = () => {
  const facts = systemFacts(useSystemReads(), Date.now());
  return (
    <SettingsSection id="system" title="System">
      <FactsPanel groups={[facts]} label="System facts" />
      <Paragraph tone="muted">Switch with coord remote; the Overlord gives the go.</Paragraph>
    </SettingsSection>
  );
};

export { SystemSection };
