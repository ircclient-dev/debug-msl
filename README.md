# debug-msl

![Sponsored by Byrnes Tech Pty Ltd](https://img.shields.io/badge/Sponsor-Byrnes%20Tech%20Pty%20Ltd-blue)

`debug-msl` is a command-line tool that allows clients to execute mSL (mIRC Scripting Language) code in any active mIRC and AdiIRC instances. This tool is particularly useful for debugging and testing DLLs in real time.

## Features

- Send mSL code to active mIRC and AdiIRC instances.
- Only affects active desktop windows (minimized-to-tray windows are not targeted).
- Simple and easy-to-use command-line interface.

## Installation

To install `debug-msl`, follow these steps:

1. Clone the repository:
    ```sh
    git clone https://github.com/your-username/debug-msl.git
    ```

2. Navigate to the project directory:
    ```sh
    cd debug-msl
    ```

3. Build the project (if required):
    ```sh
    cargo build
    ```

## Usage

To use `debug-msl`, open a command prompt and run the following command:

```sh
debug-msl.exe <mSL_code>
```

For example, to send an "echo" command to all active mIRC and AdiIRC instances, use:

```sh
debug-msl.exe echo -at Hello World!
```

## License

This project is licensed under the MIT License - see the [LICENSE.md](LICENSE.md) file for details.

## Sponsors

Special thanks to our sponsor, Byrnes Tech Pty Ltd, for their support in creating this project.

## Contributing

Contributions are welcome! Please submit a pull request or open an issue to discuss improvements.