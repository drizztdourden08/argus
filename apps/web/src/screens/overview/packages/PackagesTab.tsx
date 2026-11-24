/* @layer renderer-app @kind component */
import { Stack, Tabs, Text } from '@drizztdourden08/tessera';
import { useStored } from '../../../stored';
import { CacheTab } from './cache/CacheTab';
import { DEFAULT_PACKAGE_TAB, PACKAGE_SENTENCES, PACKAGE_TAB_KEY, PACKAGE_TABS } from './packages.constants';
import { RegistryTab } from './registry/RegistryTab';

const PackagesTab = () => {
  const [stored, setStored] = useStored(PACKAGE_TAB_KEY);
  const sub = PACKAGE_TABS.some((item) => item.id === stored) ? stored : DEFAULT_PACKAGE_TAB;
  return (
    <Stack gap="md">
      <Tabs tabs={PACKAGE_TABS} activeTab={sub} onTabChange={setStored} selectedTone="neutral" />
      <Text tone="muted">{PACKAGE_SENTENCES[sub]}</Text>
      {sub === 'cache' ? <CacheTab /> : <RegistryTab />}
    </Stack>
  );
};

export { PackagesTab };
