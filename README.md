# LX: A nicer way to list your files

## Building
First, make sure you have a local bin directory:
```
mkdir ~/.local/bin
```

Then build the project and copy the binary to the bin directory:
```
cargo build --release
cp target/release/lx ~/.local/bin/
```

Now 'lx' should be available in your path, and you can use it to list your files anywhere!
