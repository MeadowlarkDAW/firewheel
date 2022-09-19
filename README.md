# Goldenrod
[![Documentation](https://docs.rs/goldenrod/badge.svg)][documentation]
[![Crates.io](https://img.shields.io/crates/v/goldenrod.svg)](https://crates.io/crates/goldenrod)
[![License](https://img.shields.io/crates/l/goldenrod.svg)](https://github.com/MeadowlarkDAW/goldenrod/blob/main/LICENSE)

This crate is currently incomplete and just an experiment. There is no guarantee that Goldenrod will ever become something.

---

Goldenrod is an opinionated "do it yourself" UI toolkit focused on performance. It is geared towards making desktop applications with complex (but "rigid") UIs.

This toolkit was born out of the need for a high-performance UI toolkit for [Meadowlark](https://github.com/MeadowlarkDAW/Meadowlark), whos UI needs would perform innefficiently in most other UI toolkits but at the same time doesn't need a lot of the features present in modern toolkits.

If you are just looking for a feature rich & easy-to-use UI toolkit in Rust, please check out one of these UI toolkits instead (that being said, depending on your definition of "simple", you might still enjoy using Goldenrod ;) )
* [Vizia](https://github.com/vizia/vizia)
* [Iced](https://github.com/iced-rs/iced)
* [Tauri](https://github.com/tauri-apps/tauri)
* [Egui](https://github.com/emilk/egui)
* [imgui-rs](https://github.com/imgui-rs/imgui-rs)

# Goals/Non-goals

## Goals
* Rendering is done by defining isolated "layers", and then blitting those layers together to get the final output. A single widget may render to multiple layers.
* Every widget has its own dedicated area in each layer it renders to. Multiple widgets may not have overlapping render areas within the same layer. Because of this restriction, repainting only requires redrawing the areas with dirty widgets as apposed to needing to redraw the whole layer every time. Layers that don't have any dirty widgets don't need to be repainted at all (common in complex desktop app UIs with mutliple distinct panels/sections).
* Widgets cannot contain other widgets, nor can they contain references to other widgets. Every widget is solely in charge of its own logic and rendering, and only communicates with (and mutates) the outside world using custom-defined events. There is no "widget tree". You get to organize your widgets however you like.
* Layout of widgets is performed solely through a list of anchor points (with offsets to those anchor points). These anchor points are anchored to either the layer itself or to the previous widget in the list. No "Flexbox", "margins", or "padding" logic.
* A layer can layout widgets in a row, in a column, or in a grid.
* Any layer can be used as a "scroll region", and can even blit pre-rendered areas when scrolling to avoid needing to redraw the entire layer while scrolling.
* There are no "panel" widgets. You simply define all backgrounds as a series of rectangles/lines to draw in a single draw pass on a "background" layer, and then the widgets layer is blitted on top of that layer.
* No pre-determined stylesheets. You can define whatever custom styling system you want for your widgets/application.
* Has a relatively easy-to-use GPU-accelerated drawing API provided by [nanovg](https://github.com/inniyah/nanovg).
* Widgets may also use custom shaders for rendering.
* Cross-platform (only depending on OpenGL). Bring your own windowing library and event-loop!
* Hi-DPI support built-in.

## Non-goals
* Only single-line text is supported (at least for now).
* No "Flexbox"-like layout systems, so this library is not meant for web or mobile applications.
* No windowing library or event-loop logic. You must provide that yourself.

# FAQ

* What does the name "Goldenrod" mean?
    * [Goldenrod](https://en.wikipedia.org/wiki/Goldenrod) is a wildflower native to the Midwest USA. This is following a convention in the Meadowlark project of naming things after native fauna/flora from that region.

[documentation]: https://docs.rs/goldenrod/
