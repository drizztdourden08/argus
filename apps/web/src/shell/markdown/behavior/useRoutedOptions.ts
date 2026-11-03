/* @layer renderer-app @kind logic */
import { useMemo } from 'react';
import { useNavigate, useParams } from 'react-router';
import { entryOfHref } from '../../records/behavior/entry-of-href';
import { useOpenEntry } from '../../records/behavior/useOpenEntry';
import type { MarkdownOptions } from '../markdown.type';
import { boardIdRule } from './board-id-rule';
import { openLink } from './open-link';
import { proofImage } from './proof-image';

const useRoutedOptions = (): MarkdownOptions => {
  const { key } = useParams();
  const navigate = useNavigate();
  const openEntry = useOpenEntry();
  return useMemo(
    () => ({
      images: proofImage,
      linkIds: key ? boardIdRule(key) : undefined,
      onLink: (href) => {
        const entry = key ? entryOfHref(href, key) : null;
        if (entry) openEntry(entry);
        else openLink(href, (path) => void navigate(path));
      },
    }),
    [key, navigate, openEntry],
  );
};

export { useRoutedOptions };
