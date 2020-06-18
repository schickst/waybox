# waybox

Waybox is a Wayland Compositor based on [wlroots](https://github.com/swaywm/wlroots) written in Rust. It is similar to fluxbox and Openbox from the X-Window system. It will have floating windows, with some ambitions toward tiling.

This is a project to learn new stuff, do not expect this software to be useful at this point in time. In particular I want to become fluent in Rust and learn about Wayland.

## Design considerations

We make use of wlroots to do the heavy lifting, however it is written in C. Therefore we can't and won't write 100% safe rust code all the way down. Instead the goal is to write a small wrapper and encapsulate the unsafe stuff. Everything above this wrapper should then be safe rust code.

## Todos

- Make wayland-protocol generation via wayland-scanner dynamic and add more protocols


## Notes

- VS Code rust plugin with ?rls/rust-analyzer? backend has problems with code completion when using generated bindings.


## Resources

- https://wayland-book.com/
- https://github.com/swaywm/wlroots/blob/master/tinywl/tinywl.c
- https://hg.sr.ht/~icefox/swot