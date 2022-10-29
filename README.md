# Firewheel
[![Documentation](https://docs.rs/firewheel/badge.svg)][documentation]
[![Crates.io](https://img.shields.io/crates/v/firewheel.svg)](https://crates.io/crates/firewheel)
[![License](https://img.shields.io/crates/l/firewheel.svg)](https://github.com/MeadowlarkDAW/firewheel/blob/main/LICENSE)

This crate is a work in progress. It is not yet ready for any kind of use.

---

Firewheel is a low-level, barebones, "DIY" toolkit aimed at making high-performance GUIs. It is *NOT* a complete GUI framework with an extensive suite of ready-made widgets, but rather it is a toolkit to aid in building your own widgets and GUI systems.

This project was born out of the need for a high-performance GUI toolkit for [Meadowlark](https://github.com/MeadowlarkDAW/Meadowlark). Meadowlark's GUI is quite unconventional (as far as generic GUI toolkits are concerned), because it contains a whole lot of custom widgets, custom layout logic, custom rendering logic (with shaders), and unique performance optimization challenges. So in the end I decided to develop an in-house toolkit that is tailored to the needs of Meadowlark (and to my personal coding workflow).

If you are just looking for a easy-to-use/feature rich GUI toolkit in Rust, please check out one of these GUI toolkits instead:
* [Vizia](https://github.com/vizia/vizia)
* [Iced](https://github.com/iced-rs/iced)
* [Egui](https://github.com/emilk/egui)
* [Tauri](https://github.com/tauri-apps/tauri)
* [Slint](https://github.com/slint-ui/slint)
* [Druid](https://github.com/linebender/druid)
* [Relm](https://github.com/antoyo/relm)
* [gtk4-rs](https://github.com/gtk-rs/gtk4-rs)
* [gtk3-rs](https://github.com/gtk-rs/gtk3-rs)
* [imgui-rs](https://github.com/imgui-rs/imgui-rs)

# How it works

* Layer system
    * Rendering is done by defining separate "layers" (textures that widgets paint their contents onto), and then blitting those layers together to get the final output. A widget can be assigned to any layer.
    * Every widget has its own dedicated rectangular region in the layer it belongs to. A widget may not paint outside of its assigned region, and multiple widgets may not have overlapping regions within the same layer. Because of these restrictions, repainting a layer only requires redrawing the regions with dirty widgets as apposed to redrawing the entire layer every frame. Layers that don't have any dirty widgets don't need to be repainted at all (common in complex desktop app GUIs with mutliple distinct panels/sections).
    * Any layer can effectively be used as a "scroll region" by simply changing its internal offset (like a camera in a game engine). Regions that are outside the layer bounds are automatically culled from both rendering and input logic.
    * Layers are automatically packed together into as few textures as possible to save GPU memory and therefore increase performance.
* Widget organization
    * Widgets cannot contain other widgets, nor can they contain references to other widgets. Every widget is solely in charge of its own logic and rendering, and only communicates with (and mutates) the outside world using custom events.
    * There is no "widget tree" or DOM. Every widget is essentially "root level", and you get to organize them in your code however you like.
    * There are no "container" or "panel" widgets. You simply define all backgrounds as literally a series of rectangles, lines, and/or textures to draw in a single draw pass on a "background" layer, and then the widget layers are blitted on top. You can of course have as many background layers and widget layers as you want in your application.
* Layout system
    * Layout of widgets is performed via a tree of abstract rectangular regions. Each region contains an internal anchor point, a parent achor point, and an offset between those two points. No "Flexbox", "margins", or "padding" logic.
    * The size, anchors, and anchor offsets are defined manually for each region. However, a change to the size or anchor offset of a container (parent) region will automatically update the position of its child regions.
    * Only widget regions are allowed to paint to the screen. Container (parent) regions are completely abstract, and a widget region cannot be a container region.
    * Any region can be set to be explicitly visible/invisible. A region that is explicitly invisible will have it and all its children automatically culled from both rendering and input logic.
    * The user can specify whether child regions should internally be unsorted (default), sorted by x coordinate, or sorted by y coordinate. The sorted variants will allow for further scrolling and pointer input optimizations for long lists of items.
* Pointer (mouse) input logic
    * Widgets can request to receive/stop receiving pointer input events. Only the widgets that have opted-in will receive pointer input events.
    * Pointer input starts at the layer with the highest z-order value, and then it works its way down the layer list until a layer captures the event.
    * Pointer input is optimized by only invoking the layers and parent regions that contain the position of the pointer, and then walking down the region tree until a widget captures the event. Once an event is captured, walking the tree is stopped.
    * Widgets can request to "lock"/"unlock" the pointer. A widget that has locked the pointer will receive pointer input events even if the pointer is outside the bounds of the widget's assigned region, which can be used to create drag gestures. Only one widget can lock the pointer at a time.
    * Widgets can request to receive/stop receiving pointer events regardless of position, allowing for behavior such as unhighlighting a button on pointer-leave or closing a pop-up by clicking outside of it.
    * A widget can flag itself as being input-only, meaning it only listens to pointer input events and doesn't paint anything to the screen. This can be used to create drag handles or drag-and-drop targets.
    * Keyboard modifiers are sent along with each pointer input event.
* Keyboard input logic
    * Widgets can request to receive/stop receiving keystroke events. Only the widgets that have opted-in will receive keystroke input events.
    * A widget can also request to receive/stop recieving text composition events. Only one widget at a time can be assigned to receive text composition events.
* Animation logic
    * A widget can request to receive/stop recieving the "animation" input event, which is an event sent every frame. Only the widgets that have opted-in will receive this event.
* Styling & custom drawing
    * There are no pre-determined stylesheets. You can define whatever custom styling system you want for your widgets/application.
    * Firewheel has a relatively easy-to-use GPU-accelerated drawing API provided by [femtovg](https://github.com/femtovg/femtovg).
    * Widgets may also use custom shaders for rendering.
* Portability
    * Cross-platform support (only depending on OpenGL (ES) 3.0+). Bring your own windowing library and event-loop!
    * Hi-DPI support built-in. Firewheel uses logical pixel coordinates.

# Non-goals
* No multi-line text (at least for now).
* No extensive suite of ready-made widgets. Only a few basic ones will be included such as buttons, toggle buttons, labels, spinners, scrollbars, separators, drop-down menus, and single-line text input.
* No "Flexbox"-like layout systems, so this library is not meant for web or mobile applications.
* No windowing library or event-loop logic. You must provide that yourself. (Look at the examples for how to do this.)
* This toolkit makes no gaurantees that your GUI will perform optimally or correctly if you don't know what you are doing. Using this toolkit requires some knowledge of how to optimally create layers and regions for a GUI. (I may create a guide for this later.)

# FAQ

* Why the name "Firewheel"?
    * [Firewheel](https://en.wikipedia.org/wiki/Gaillardia_pulchella) is a wildflower native to the Midwest USA. This is following a convention in the Meadowlark project of naming things after native fauna/flora from that region (or things related to nature).
    * The vibrant colors of this wildflower represent drawing elements on the screen, and "fire" alludes to the high-performance goals of this toolkit.

[documentation]: https://docs.rs/firewheel/
