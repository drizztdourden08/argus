/* @layer renderer-app @kind logic */
import type { Entry } from '../../../../data/types';
import type { ImageGroup } from '../OwnerFeed.type';

const optionTitle = (entry: Entry, option: number): string => {
  const label = entry.options?.[option - 1];
  return label ? `Option ${option}: ${label}` : `Option ${option}`;
};

const imageGroups = (entry: Entry): ImageGroup[] => {
  const images = entry.images ?? [];
  const whole = images.filter((image) => !image.option);
  const groups: ImageGroup[] = whole.length > 0 ? [{ key: 'entry', title: null, shots: whole }] : [];
  const options = [...new Set(images.flatMap((image) => (image.option ? [image.option] : [])))].sort((a, b) => a - b);
  options.forEach((option) => {
    groups.push({ key: `option-${option}`, title: optionTitle(entry, option), shots: images.filter((image) => image.option === option) });
  });
  return groups;
};

export { imageGroups };
