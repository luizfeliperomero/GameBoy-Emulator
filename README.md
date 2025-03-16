# Game Boy Emulator
## Usage
It is possible to run the emulator in two modes: **default** and **debug**.
Both modes runs the emulator with the pre-allocated ROM (currently _Super Mario Land_)
### Default
In this mode, the emulator runs normally without additional debugging information.\
To run the emulator in default mode, use the following command:
 ```sh
cargo run
```
### Debug
The debug mode provides runtime information about the current state of the CPU and memory internals.\
To run the emulator in debug mode, enable the debug feature by using the following command:
 ```sh
cargo run --features debug
```
