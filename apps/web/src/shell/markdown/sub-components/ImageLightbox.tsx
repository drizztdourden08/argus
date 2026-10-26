/* @layer renderer-app @kind component */
import { MediaViewer } from '@drizztdourden08/tessera';
import { imageItems } from '../behavior/image-items';
import type { ImageLightboxProps } from '../markdown.type';

const ImageLightbox = ({ image, onClose }: ImageLightboxProps) => <MediaViewer open={image !== null} items={imageItems(image)} onClose={onClose} />;

export { ImageLightbox };
