# LX: A nicer way to list your files

## Building:

First, build the project:
```
cargo build --release
```

The compiled executable will be at target/release/lx-cli. You can install it by moving or linking it to a directory that is already in your $PATH.
Move the binary and rename it to lx:
```
# Make sure ~/.local/bin is in your $PATH!
mv target/release/lx-cli ~/.local/bin/lx
```

Best Practice (Cargo Install): You can also use cargo install, which places the binary in your $HOME/.cargo/bin directory (which is usually added to your $PATH when installing Rust).
```
# Run this from the root of your project (lx-cli directory)
cargo install --path . --force
# The command will be named 'lx-cli' by default, you can rename it
# in Cargo.toml if you want the final name to be just 'lx'.
```

Now, you can run your new Rust program from any directory in your terminal!
