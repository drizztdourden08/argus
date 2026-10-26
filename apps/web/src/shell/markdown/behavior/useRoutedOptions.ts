/* @layer renderer-app @kind logic */
import { useMemo } from 'react';
import { useNavigate, useParams } from 'react-router';
import type { MarkdownOptions } from '../markdown.type';
import { boardIdRule } from './board-id-rule';
import { openLink } from './open-link';
import { proofImage } from './proof-image';

const useRoutedOptions = (): MarkdownOptions => {
  const { key } = useParams();
  const navigate = useNavigate();
  return useMemo(
    () => ({ images: proofImage, linkIds: key ? boardIdRule(key) : undefined, onLink: (href) => openLink(href, (path) => void navigate(path)) }),
    [key, navigate],
  );
};

export { useRoutedOptions };
