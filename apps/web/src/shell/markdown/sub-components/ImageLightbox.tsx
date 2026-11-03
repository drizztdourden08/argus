/* @layer renderer-app @kind component */
import { MediaViewer } from '@drizztdourden08/tessera';
import { imageItems } from '../behavior/image-items';
import type { ImageLightboxProps } from '../markdown.type';

const ImageLightbox = ({ image, gallery, onClose }: ImageLightboxProps) => {
  const items = imageItems(image, gallery);
  return <MediaViewer key={image?.src ?? ''} open={image !== null} items={items} defaultIndex={Math.max(0, items.findIndex((item) => item.src === image?.src))} onClose={onClose} />;
};

export { ImageLightbox };
