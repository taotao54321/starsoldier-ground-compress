# NES Star Soldier ground data compression

## Usage

```
# extract ground.bin from StarSoldier.nes
./extract

# uncompress ground.bin
cargo run --bin decode -- ground.bin raw.bin

# compress raw.bin
cargo run --bin encode -- raw.bin compressed.bin
```
