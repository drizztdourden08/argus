/* @layer renderer-app @kind test */
import { act } from '@testing-library/react';
import { minutesAgo } from './minutes-ago';
import { renderAt } from './render-at';
import { seedBoard } from './seed-board';
import { snapshot } from './snapshot';
import { waitForStream } from './wait-for-stream';

type Docs = Record<string, object>;

const openRotp = async (seed: { wakes?: Docs; registry?: Docs; threads?: Docs } = {}): Promise<void> => {
  const { sources } = renderAt('/p/rotp/');
  await waitForStream(sources);
  const { wakes, ...rest } = seed;
  act(() => {
    if (wakes) sources[0]?.emit('snapshot', snapshot('rotp', 'wakes', wakes));
    seedBoard(sources[0], { meta: { board: { project: 'rotp', name: 'RotP', updatedAt: minutesAgo(0) } }, ...rest });
  });
};

export { openRotp };
