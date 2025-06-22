# Low Level Triangulator

This is an implementation of the monotone polygon triangulation algorithm, programming entirely in Rust.

Currently it can break any "simple" polygon into triangles, and renders it to the screen with a low level graphics library called wgpu.

The algorithm breaks down polygon into monotone sections. Each section consists of two strings of vertices joined at each end, which do not cross one another, and strictly progress from left to right from each vertex to the next.

These monotones are much easier to break down into triangles which can easily be rendered by the graphics library.

In the future I'd like it to break down more complex polygons like ones with holes, and for it to be able to fix self-intersection.
