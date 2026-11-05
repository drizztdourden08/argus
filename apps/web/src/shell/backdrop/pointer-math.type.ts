/* @layer renderer-app @kind types */
type PointerPoint = { readonly x: number; readonly y: number };

type DriftFrame = { readonly point: PointerPoint; readonly blob: PointerPoint; readonly blobWeight: number };

export type { DriftFrame, PointerPoint };
