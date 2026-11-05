# Shader attribution

The mesh backdrop shader in `apps/web/src/shell/backdrop/mesh-shader.constants.ts` is adapted from
the mesh gradient fragment shader of Paper Shaders (<https://github.com/paper-design/shaders>),
licensed under the Apache License 2.0.

Changes made here:

- The colour spots are pushed away from the pointer (`u_pointer`, `u_repel`, `u_repelRadius`).
- Each spot shifts by its own depth from the pointer offset (`u_parallax`).
- The grain follows the canvas aspect (`u_aspect`).

The vertex shader and the sizing uniforms are Paper's own.
