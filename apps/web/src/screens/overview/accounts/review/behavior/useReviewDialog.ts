/* @layer renderer-app @kind logic */
import { useCallback, useState } from 'react';
import type { DialogState, ReviewDialog } from '../review.type';

const useReviewDialog = (): DialogState => {
  const [dialog, setDialog] = useState<ReviewDialog | null>(null);
  const close = useCallback(() => setDialog(null), []);
  return { dialog, open: setDialog, close };
};

export { useReviewDialog };
