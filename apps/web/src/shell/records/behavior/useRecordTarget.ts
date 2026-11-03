/* @layer renderer-app @kind logic */
import { useLocation, useNavigate, useParams } from 'react-router';
import type { RecordRef, RecordTarget } from '../records.type';
import { recordHash } from './record-hash';
import { recordHref } from './record-href';
import { useOpenEntry } from './useOpenEntry';

const useRecordTarget = (ref: RecordRef | null): RecordTarget | null => {
  const here = useParams().key ?? null;
  const { search } = useLocation();
  const navigate = useNavigate();
  const openEntry = useOpenEntry();
  const href = ref ? recordHref(ref, here) : null;
  if (!ref || !href) return null;
  if ((ref.board ?? here) !== here) return { href, go: () => void navigate(href) };
  if (ref.kind === 'entry') return { href, go: () => openEntry(ref.id) };
  return {
    href,
    go: () => {
      openEntry(null);
      void navigate({ search, hash: recordHash(ref) }, { replace: true });
    },
  };
};

export { useRecordTarget };
