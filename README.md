<center><img src=".github/banner.png" /></center>

> A simple, very minimal Minecraft server implementation in Rust.

For a simple Minecraft server that isn't supposed to do much (for example, a _limbo_ server as a fallback in a network of servers), usual server implementations based on Mojang's official server provide a lot of overhead.

That's why I've decided to write my own server implementation in Rust, which should be a lot faster and more efficient in handling very specific use-cases.

**Note:** This server is mostly meant as a learning project, not intended to be used in production. Because of this, bugs exists and the project is mostly unfinished.

## Development

You will need to have Rust installed along with it's package manager Cargo (which should be installed by default).

After that, you can run the server using Cargo:

```bash
cargo run
```

It will automatically generate a configuration file (`limbo.toml`) if it does not exist yet, or use the existing one.

## Credits

- [wiki.vg](https://wiki.vg/) for being the *best* Minecraft protocol-related resource out there.
- [feather](https://github.com/feather-rs/feather) for inspiration on how best to implement some specific features.
- [BomBaryGamer](https://github.com/BomBardyGamer) for providing some of the NBT files required for the client to log in (specifically [`dimension_codec`](https://gist.github.com/BomBardyGamer/c075a7a34b51f2df9d5aabdd2a762f4f)).
