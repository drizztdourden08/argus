/* @layer renderer-app @kind component */
import { Callout, Stack } from '@drizztdourden08/tessera';
import { useReviewDeck } from '../behavior/useReviewDeck';
import { choiceTotals, decisionsPayload, editTotals } from '../model';
import type { ReviewBodyProps } from '../review-props.type';
import { BulkBars } from './BulkBars';
import { PendingPanel } from './PendingPanel';
import { ReviewDialogs } from './ReviewDialogs';
import { ReviewEmpty } from './ReviewEmpty';
import { ReviewTable } from './ReviewTable';
import { ReviewToolbar } from './ReviewToolbar';

const ReviewBody = (props: ReviewBodyProps) => {
  const { data, loading, error, onRescan, onBack, onApplied } = props;
  const deck = useReviewDeck(data, onApplied);
  const { rows, accounts, shown, choices, staged, picks, dialog } = deck;
  const sendApply = () => {
    deck.submit.apply(decisionsPayload(rows, choices.decisions), deck.edits);
    dialog.close();
  };
  return (
    <Stack gap="md">
      <ReviewToolbar
        accounts={accounts}
        counts={data?.counts}
        edits={deck.edits.length}
        filter={deck.filter}
        loading={loading}
        applyCount={deck.count}
        onFilter={deck.setFilter}
        onBack={onBack}
        onRescan={onRescan}
        onCleanup={() => dialog.open({ kind: 'cleanup' })}
        onApply={() => dialog.open({ kind: 'apply' })}
      />
      {error && <Callout tone="danger">{error}</Callout>}
      <PendingPanel items={staged.items} names={deck.names} changed={deck.changed} onRemove={staged.remove} />
      <BulkBars
        accounts={accounts}
        rows={rows}
        selection={picks.selection}
        onStage={staged.add}
        onMove={(account, ids) => dialog.open({ kind: 'move', account, ids })}
        onClear={picks.clear}
      />
      <ReviewEmpty loaded={data !== null} total={rows.length} shown={shown.length} />
      {shown.length > 0 && <ReviewTable accounts={accounts} rows={shown} env={deck.env} />}
      <ReviewDialogs
        dialog={dialog.dialog}
        accounts={accounts}
        rows={rows}
        choiceLines={choiceTotals(rows, choices.decisions)}
        editLines={editTotals(deck.edits)}
        count={deck.count}
        applying={deck.submit.busy}
        onClose={dialog.close}
        onStage={staged.add}
        onApply={sendApply}
      />
    </Stack>
  );
};

export { ReviewBody };
