/* @layer renderer-app @kind test */
import { act } from '@testing-library/react';
import type { FakeSource } from '../../helpers/fake-source';
import { seedBoard } from '../../helpers/seed-board';
import { liveStream } from '../../helpers/wait-for-stream';

const seedProject = async (sources: FakeSource[], scope: string, docs: Record<string, Record<string, object>> = {}): Promise<void> => {
  await act(async () => undefined);
  const source = await liveStream(sources, scope);
  act(() => seedBoard(source, docs, scope));
};

export { seedProject };
