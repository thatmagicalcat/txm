<div align="center">
  <h1>TXM</h1>
  <p>TXM (Terminal TeX Math) is a math rendering engine with LaTeX support.</p>
</div>

# Screenshots:
![s1](./screenshots/1.png)
![s2](./screenshots/2.png)
![s3](./screenshots/3.png)
![s4](./screenshots/4.png)
![s5](./screenshots/5.png)

### Quick run using nix
```
nix run github:thatmagicalcat/txm -- "E = mc^2"
```
Requires [Nix](https://nix.dev/install-nix) with flakes enabled.

# Installation
### Arch Linux (AUR)
Install `txm-git` using an AUR helper like `yay` or `paru`:
```bash
yay -S txm-git
```

Or install it manually:
```bash
git clone https://aur.archlinux.org/txm-git.git
cd txm-git
makepkg -si
```

### Using Cargo (Rust)
```
$ cargo install txm
```
Or
```
$ cargo install --git https://github.com/thatmagicalcat/txm
```

## License
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))
