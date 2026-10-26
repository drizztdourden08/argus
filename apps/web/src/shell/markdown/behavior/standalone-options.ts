/* @layer renderer-app @kind logic */
import type { MarkdownOptions } from '../markdown.type';
import { openLink } from './open-link';
import { proofImage } from './proof-image';

const standaloneOptions: MarkdownOptions = {
  images: proofImage,
  linkIds: undefined,
  onLink: (href) => openLink(href, (path) => window.location.assign(path)),
};

export { standaloneOptions };
