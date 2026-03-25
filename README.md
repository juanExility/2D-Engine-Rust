# 🚀 engine_2d (Stellar Dodge Edition)

A lightweight, high-performance 2D game engine built with **Rust** 🦀. 
This project explores the **Entity Component System (ECS)** architecture and software-based rendering using the `minifb` library.

> **Current Status:** Under active exploration and reverse engineering.
> Developed during "Geography World Tours" and "German Exam Recovery" sessions. 🕵️‍♂️

## 🛠️ Tech Stack & Architecture

- **Language:** Rust (Memory safe, high-performance, Assembly-adjacent)
- **Core:** Custom ECS (`ecs.rs`) for optimized data management.
- **Rendering:** Software pixel buffer rendering (`renderer.rs`) with custom font mapping.
- **Physics:** Vector-based math and AABB collision detection (`math.rs`).
- **Hardware:** Tested on a Ryzen 7 7800X3D + RTX 5070 beast. 💻

## 🏗️ Project Structure

- `src/engine/ecs.rs`: The "Brain" - manages entity-component relationships.
- `src/engine/renderer.rs`: The "Painter" - handles pixel-perfect drawing.
- `src/main.rs`: **Stellar Dodge** - The demo game showcasing the engine capabilities.

## 🕹️ How to Run

1. Make sure you have the Rust toolchain installed.
2. Clone the repo.
3. Run the following command in your terminal (avoid `rm -rf /*` please):
   ```bash
   cargo run --release
