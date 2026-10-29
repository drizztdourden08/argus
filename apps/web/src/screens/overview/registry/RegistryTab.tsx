/* @layer renderer-app @kind component */
import { Callout, EmptyState, Stack } from '@drizztdourden08/tessera';
import { readRegistry, REMOTE_POLL_MS, useRemoteRead } from '../../../data/remote';
import { Panel } from '../../../shell';
import { NO_BUILDS, NO_CONSUMERS, READING, REDIRECTED_TEXT } from './registry.constants';
import { BuildsTable } from './sub-components/BuildsTable';
import { ConsumersTable } from './sub-components/ConsumersTable';

const RegistryTab = () => {
  const { data, error } = useRemoteRead(readRegistry, REMOTE_POLL_MS);
  if (!data) return error ? <Callout tone="danger">{`The local registry could not be read: ${error}.`}</Callout> : <EmptyState message={READING} />;
  const consumers = data.consumers.filter((consumer) => consumer.on);
  return (
    <Stack gap="md">
      {data.scopeRedirected && <Callout tone="warning">{REDIRECTED_TEXT}</Callout>}
      <Panel title="Local builds" note={data.builds.length > 0 ? `${data.builds.length} ${data.builds.length === 1 ? 'package' : 'packages'}` : undefined}>
        {data.builds.length > 0 ? <BuildsTable builds={data.builds} /> : <EmptyState message={NO_BUILDS} />}
      </Panel>
      <Panel title="Consumers switched on" note={consumers.length > 0 ? `${consumers.length}` : undefined}>
        {consumers.length > 0 ? <ConsumersTable consumers={consumers} /> : <EmptyState message={NO_CONSUMERS} />}
      </Panel>
    </Stack>
  );
};

export { RegistryTab };
