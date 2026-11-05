/* @layer renderer-app @kind types */
type Mount = { paperShaderMount?: { setUniforms: (uniforms: Record<string, number | number[]>) => void } } | null;

export type { Mount };
