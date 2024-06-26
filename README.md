# egui_glfw_gl2
Egui backend implementation for GLFW and OpenGL

[![Latest version](https://img.shields.io/crates/v/egui_glfw_gl2.svg)](https://crates.io/crates/egui_glfw_gl2)
![MIT](https://img.shields.io/badge/license-MIT-blue.svg)

![Example screenshot](/media/screenshot.jpg)

This is a backend implementation for [Egui](https://github.com/emilk/egui) that can be used with Rust bindings for [GLFW](https://github.com/PistonDevelopers/glfw-rs) and [OpenGL](https://github.com/brendanzab/gl-rs).

Since the [egui_glfw_gl](https://github.com/cohaereo/egui_glfw_gl) has not been updated for a long time, I have updated the dependencies to the latest version and made the following things:
1. Refactored some parts of the code.
2. Fixed the issue in the old project's demo where the "actual size" of the window did not match the visible size.
3. Implemented the window scroll event, you can now use the mouse wheel in the egui window.
4. Implemented copy and paste functionality.
5. The mouse cursor is now properly displayed by using crates "winapi".

## Example
I have made an example to demonstrate the usage of egui_glfw_gl. To run the example, run the following:
```
cargo run --example demo
```

## Known issues
- Due to the addition of the cursor's icon part, project is currently not compatible with Linux and MacOS.

## Credits
egui_glfw_gl2 is based off [egui_glfw_gl](https://github.com/cohaereo/egui_glfw_gl), created by [cohae](https://github.com/cohaereo)

The project's code heavily references the implementations of [winit](https://github.com/rust-windowing/winit) and [egui_vulkano](https://github.com/derivator/egui_vulkano). Many thanks to them.

## Update
### 0.1.2 (2024-3-31)
- Significantly refactored the project.
- Now OpenGL uses version 330, and the method of uploading vertex data has been modified, which theoretically improves rendering performance.
### 0.1.1 (2024-3-27)
- Breaking change: Switched OpenGL binding crate from [gl](https://crates.io/crates/gl) to [gl33](https://crates.io/crates/gl33) (as the function and variable names in gl33 are consistent with C++).
- Update: Updated the egui dependency to the latest version (0.27.0).
### 0.1.0 (2024-3-24)
- Updated the egui dependency to 0.26.2.
- Refactored some parts of the code.
