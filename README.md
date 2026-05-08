# gomoku-rm

A Gomoku (五子棋) game for the **reMarkable 2** tablet, designed to run inside
the [xovi / AppLoad](https://github.com/asivery/rm-appload) framework.

- 19×19 board (Go-standard intersections, A–T × 1–19 labels, 9 star points)
- Play vs AI **or** local 2-player on the same device
- Self-implemented Alpha-Beta engine with iterative deepening, candidate
  pruning (Chebyshev radius 2), and a 3-second time budget
- 3 difficulty levels (Easy / Medium / Hard → search depth 2 / 4 / 6)
- Bilingual UI scaffolding (English / Simplified Chinese)
- Auto-saves after every move; resume from main menu
- Pure Rust on top of [`libremarkable`](https://crates.io/crates/libremarkable)

## Architecture

```
src/
├── main.rs           # 30 fps scene loop + Transition state machine
├── canvas.rs         # libremarkable Framebuffer wrapper
├── board.rs          # 19×19 cells, history, win detection
├── i18n.rs           # Lang + t(key) lookup
├── config.rs         # Persistent difficulty / language / ai_side
├── savestate.rs      # YAML save/load for in-progress games
├── engine/
│   ├── mod.rs        # Engine trait (lets us swap in mintaka / rapfi later)
│   ├── shape.rs      # Pattern recognition (five / open four / open three / …)
│   ├── evaluate.rs   # Sum pattern scores across rows / cols / diagonals
│   └── alphabeta.rs  # Negamax + alpha-beta + iterative deepening
└── scene/
    ├── mod.rs        # Scene trait + Transition
    ├── menu.rs       # MainMenuScene
    ├── game.rs       # GameScene (board + AI + buttons)
    └── settings.rs   # SettingsScene
```

## Build (Apple Silicon Mac)

`cargo zigbuild` is preferred over `cross` because Apple Silicon hosts
cannot natively install the x86_64-linux toolchain that `cross` requires.

```sh
brew install zig
cargo install cargo-zigbuild
rustup target add armv7-unknown-linux-gnueabihf
cargo zigbuild --target armv7-unknown-linux-gnueabihf --release
```

The result lives at
`target/armv7-unknown-linux-gnueabihf/release/gomoku-rm` (~2 MB ARM ELF).

### ⚠️ ELF e_machine guard

Some Rust release pipelines (notably the
[`chessmarkable 0.8.1-1`](https://github.com/LinusCDE/chessmarkable/releases/tag/0.8.1-1)
release) emit ARM binaries with `e_machine = 0` (EM_NONE), which the kernel
refuses to `execve` with `Exec format error`. `packaging/deploy.sh` checks
byte 18 and patches if needed:

```sh
xxd -s 18 -l 2 ./gomoku-rm           # must print "2800"
printf '\x28\x00' | dd of=./gomoku-rm bs=1 seek=18 count=2 conv=notrunc
```

## Deploy

```sh
RM_PASS='<device password>' ./packaging/deploy.sh
```

This:
1. Builds via `cargo zigbuild`
2. Verifies (and patches if necessary) the ELF `e_machine` field
3. `scp`s the binary + `external.manifest.json` (+ `icon.png` if present) to
   `/home/root/xovi/exthome/appload/gomoku/` on the rM2 (USB IP `10.11.99.1`)
4. `chmod +x` the binary

The default rM2 USB SSH password is shown on-device under
**Settings → Help → Copyrights and software licenses**.

## AppLoad manifest

Key environment variables (see `packaging/external.manifest.json`):

| Variable | Why |
|---|---|
| `LD_PRELOAD=/home/root/shims/qtfb-shim.so` | Translates `/dev/fb0` ioctls to the AppLoad framebuffer |
| `LIBREMARKABLE_FB_DISFAVOR_INTERNAL_RM2FB=1` | libremarkable 0.7 defaults to swtfb on rM2; force ioctl mode so `qtfb-shim` can hook in |
| `QTFB_SHIM_MODE=N_RGB565` | Native 565 framebuffer mode (matches device + libremarkable) |
| `QTFB_SHIM_INPUT_MODE=NATIVE` | Use real `/dev/input/event*` devices |

This is the same shim configuration used by KOReader's AppLoad package.

## Engine notes

The Alpha-Beta engine evaluates positions by encoding each line (row, col,
diagonal, anti-diagonal) as an ASCII string of `O` (mine), `X` (opponent or
edge), `_` (empty), and counting overlapping windowed substring matches of
patterns ranging from `OOOOO` (1 000 000) down to `_OO_` (100). Patterns
inspired by [lihongxun945/gobang](https://github.com/lihongxun945/gobang).

Move ordering uses a 1-ply evaluation pass on candidates; iterative
deepening ensures we always have a "completed" depth's result when the
3-second budget runs out.

`Engine` is a trait — future versions could swap in
[mintaka](https://github.com/junghyun397/mintaka) (Rust PVS+VCF) or
[rapfi](https://github.com/dhbloo/rapfi) (Gomocup champion, via Gomocup
protocol IPC) without touching the rest of the codebase.

## Known limitations

- **CJK rendering**: libremarkable's embedded font is Latin-only.
  Switching language to 中文 in Settings updates all labels semantically,
  but Chinese glyphs may render as missing-glyph boxes until proper
  CJK font loading is added (likely via direct rusttype + LXGW).
- **AI strength**: ~amateur 1-3 dan at depth 6 with the budget.
  Stronger play needs transposition tables, killer moves, or swapping in
  mintaka / rapfi via the `Engine` trait.

## License

Personal project; no license declared yet.
