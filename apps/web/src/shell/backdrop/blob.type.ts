/* @layer renderer-app @kind types */
import type { PointerPoint } from './pointer-math.type';

type BlobState = { readonly point: PointerPoint; readonly presence: number };

export type { BlobState };
