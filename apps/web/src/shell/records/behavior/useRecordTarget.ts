/* @layer renderer-app @kind logic */
import { useNavigate, useParams } from 'react-router';
import type { RecordRef, RecordTarget } from '../records.type';
import { recordHref } from './record-href';
import { useRecordPane } from './useRecordPane';

const useRecordTarget = (ref: RecordRef | null): RecordTarget | null => {
  const here = useParams().key ?? null;
  const navigate = useNavigate();
  const pane = useRecordPane();
  const href = ref ? recordHref(ref, here) : null;
  if (!ref || !href) return null;
  if ((ref.board ?? here) !== here) return { href, go: () => void navigate(href) };
  return { href, go: () => pane.open(ref.code, ref.tab) };
};

export { useRecordTarget };
