/* @layer renderer-app @kind logic */
import { useMemo } from 'react';
import { useNavigate, useParams } from 'react-router';
import { openOfHref } from '../../records/behavior/open-of-href';
import { useRecordPane } from '../../records/behavior/useRecordPane';
import type { MarkdownOptions } from '../markdown.type';
import { boardIdRule } from './board-id-rule';
import { openLink } from './open-link';
import { proofImage } from './proof-image';

const useRoutedOptions = (): MarkdownOptions => {
  const { key } = useParams();
  const navigate = useNavigate();
  const pane = useRecordPane();
  return useMemo(
    () => ({
      images: proofImage,
      linkIds: key ? boardIdRule(key) : undefined,
      onLink: (href) => {
        const target = key ? openOfHref(href, key) : null;
        if (target) pane.open(target.code, target.tab);
        else openLink(href, (path) => void navigate(path));
      },
    }),
    [key, navigate, pane],
  );
};

export { useRoutedOptions };
