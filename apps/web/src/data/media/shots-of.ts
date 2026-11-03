/* @layer renderer-app @kind logic */
import type { Shot, ShotSource } from './media.type';

const newestFirst = (a: Shot, b: Shot): number => b.at.localeCompare(a.at) || a.item.localeCompare(b.item, undefined, { numeric: true }) || b.n - a.n;

const shotsOf = (items: readonly ShotSource[]): Shot[] =>
  items
    .filter((item) => !item.deletedAt)
    .flatMap((item) =>
      (item.images ?? []).map((image) => ({ key: `${item.id}:${image.n}`, item: item.id, title: item.title, n: image.n, url: image.url, caption: image.caption, at: image.at ?? '' })),
    )
    .sort(newestFirst);

export { shotsOf };
