## free6
mee6 leveling but its foss

### what u need to run the thing
rust + cargo
nodejs
postgres
redis

### run the thing
1. start the gateway `cargo run --bin gateway --release`
2. start slash commands `yarn compile && node build/src/index.js`
3. start leveling `cargo run --bin leveling --release`
