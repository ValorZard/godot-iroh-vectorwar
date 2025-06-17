# Rust Template for Godot

This is a template project for using Rust in Godot, created based on the official [Godot-Rust](https://godot-rust.github.io/book/intro/hello-world.html) guide. It serves as a starting point for developers who want to integrate Rust into their Godot projects for better performance and type safety.

## Features
- Template project to get started with Godot and Rust.
- Configured to work with Godot Engine and the [Godot Rust bindings](https://github.com/godot-rust/gdext).
- Provides a simple "Hello, World!" example to demonstrate how to integrate Rust code into Godot.
- Setup is based on the [Hello World](https://godot-rust.github.io/book/intro/hello-world.html) tutorial from the official Godot-Rust book.

## Requirements
- **Godot Engine** version 4.4 or later.
- **Rust** installed. You can download it from the official website: https://www.rust-lang.org/
- **Cargo** – the Rust package manager, which is included when installing Rust.

## Installation

1. Clone this repository or download the ZIP.

2. Make sure you have Godot and Rust set up correctly.

3. Navigate to the `rust-template-godot` project folder and open the `rust-template` project with Godot.

4. Build the Rust code:
   - In the terminal, go to the project `rust` directory and run:
     ```
     cargo build
     ```

5. Run the project from Godot.

## Usage

Once everything is set up, you can start adding your own Rust code into the project. The template includes a simple example that prints "Hello, World!" to the Godot console, and adds a `Player` class based on Sprite2D. This can be extended to your game logic.

To modify the Rust code:
1. Open `src/lib.rs`.
2. Add your custom functionality or game logic written in Rust.
3. After making changes, rebuild your project using `cargo build` and test the integration in Godot.

### Visual Studio Code

If you are working with VS Code, I recommend you to use the `rust-analyzer` extension and setting the `Check: Command` to `build`. This enables you library to be compiled each time you save your files, allowing for fast changes to be applied inside the Godot Editor without having to compile them in the terminal yourself each time.

> This is only useful in Godot 4.2+ since it allows to import the changes without reloading the project. Since this template aims for 4.4+, this should not be a problem. Keep this in mind if you try a lesser version though.

## Project Structure

- `rust`: The Rust directory for writing code.
- `rust-template`: The Godot project directory, where scenes and assets are located.
- `README.md`: This file.
- `LICENSE` The MIT license

## Troubleshooting

- If you encounter issues with Rust not building, ensure your environment is correctly configured by following the steps in the official [Godot-Rust Book](https://godot-rust.github.io/book/intro/hello-world.html).
- For specific issues with the Godot-Rust bindings, refer to the official [GitHub repository](https://github.com/godot-rust/gdext) or consult the community forums.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for more details.

The godot-rust Ferris icon was obtained from [their repository](https://github.com/godot-rust/assets) and its licence's details are explained [here](https://github.com/godot-rust/assets/blob/master/asset-licenses.md).

## Acknowledgments

- [Godot Engine](https://godotengine.org/)
- [Godot Rust](https://github.com/godot-rust/gdext) for their fantastic work on integrating Rust with Godot.
