# Bevy Clouds ☁️

A volumetric cloud rendering system built with the **Bevy Engine** and **WGSL**, focusing on mathematical accuracy and efficient volume traversal.

[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)
![Bevy Version](https://img.shields.io/badge/bevy-0.15-orange.svg)
![Rust](https://img.shields.io/badge/rust-1.93+-red.svg)

## Overview

> **Note**: This project is currently in an early **experimental state**. The implementation is a work-in-progress and is provided "as-is" for research and demonstration purposes. Performance and stability may vary across different hardware configurations.

This project implements a real-time volumetric cloud system based on the mathematical principles outlined in Victor Wang's 2005 Cornell research, *"Physically Based Real-Time Modeling and Rendering of Volumetric Clouds"*. 

The system utilizes **Raymarching** and **Shader-side Worley Noise** to generate organic cloud structures. By calculating noise procedurally within the shader, the implementation maintains a minimal memory footprint while providing high visual detail across the volume.

## Key Features

*   **Volumetric Traversal**: Implements the Slab Method for efficient ray-box intersection and volume traversal.
*   **Procedural Worley Noise**: Real-time cellular noise generation directly in the fragment shader for billowy cloud structures.
*   **Physically-Based Absorption**: Utilizes the Beer-Lambert Law for light transmittance and depth-based shading.
*   **Real-time Tuning**: Integrated `bevy_egui` dashboard for live adjustment of density, thresholds, and raymarching step counts.
*   **Custom Orbit Camera**: A low-dependency camera system designed for precise volume inspection.

## Mathematical Foundation

The rendering loop approximates the standard volume rendering equation:

$$ L(p, \omega) = \int_{0}^{D} e^{-\tau(s)} \cdot \rho(s) \cdot L_{in}(s) \, ds $$

Where:
*   **$\tau(s)$**: Optical depth calculated via Beer's Law.
*   **$\rho(s)$**: Density sampled from procedural 3D Worley noise.
*   **$L_{in}(s)$**: In-scattering approximation (currently height-gradient based).

## Getting Started

### Prerequisites

*   **Rust**: v1.93 or higher
*   **Hardware**: Supports Metal (macOS), Vulkan, or DX12.

### Running the Project

```bash
git clone https://github.com/YOUR_USERNAME/bevy-clouds.git
cd bevy-clouds
cargo run --release
```

## Controls

| Action | Input |
| :--- | :--- |
| **Orbit** | Left Click + Drag |
| **Tuning** | Use the "Cloud Settings" UI panel |

## Roadmap

- [x] Initial Raymarching Pipeline
- [x] Shader-side Worley Noise
- [x] Runtime Parameter Tuning
- [ ] `bevy_atmosphere` Integration (Procedural Sky)
- [ ] Henyey-Greenstein Phase Function (Sun Scattering)
- [ ] Temporal Super-Sampling

## Credits

Inspired by the research of Victor Wang (Cornell PCG).

---
*Licensed under the GNU General Public License v3.0.*
