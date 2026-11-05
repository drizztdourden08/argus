/* @layer renderer-app @kind test */
import { cleanup, render, screen } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { RecordPane } from '../../src/shell/records/pane/RecordPane';
import type { RecordRef } from '../../src/shell/records/records.type';

vi.mock('../../src/shell/records/pane/board/BoardPane', async () => {
  const { Text } = await import('@drizztdourden08/tessera');
  return { BoardPane: () => <Text>board pane</Text> };
});
vi.mock('../../src/shell/records/pane/ItemRecordPane', async () => {
  const { Text } = await import('@drizztdourden08/tessera');
  return { ItemRecordPane: () => <Text>item pane</Text> };
});

afterEach(cleanup);

const ref = (collection: RecordRef['collection'], kind: string, code: string): RecordRef => ({ kind, collection, code, id: code, board: null });

describe('I-112 P5 the record pane loader', { tags: ['critical'] }, () => {
  it('opens the board pane for a board item', () => {
    render(<RecordPane boardKey="_portfolio" target={ref('boards', 'board', 'BD-1')} />);
    expect(screen.getByText('board pane')).toBeTruthy();
  });

  it('keeps the item pane for every other collection', () => {
    render(<RecordPane boardKey="argus" target={ref('items', 'task', 'I-1')} />);
    expect(screen.getByText('item pane')).toBeTruthy();
  });
});
