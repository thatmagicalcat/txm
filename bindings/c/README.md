<div align="center">
  <h1>TXM.c</h1>
  <p>C/C++ bindings for TXM.</p>
</div>

### Example

```c
#include "txm.h"
#include <stdio.h>

int main() {
    char *out = txm_render("E = mc^2");
    if (out) {
        printf("%s\n", out);
        txm_free_string(out);
    }
    return 0;
}
```

### Build

Build the shared / static library:

```bash
cd bindings/c
cargo build --release
```

The compiled library will be at `target/release/libtxm_c.{so,dylib,a}`.

Link it with your C/C++ program:

```bash
# Dynamic linking (set LD_LIBRARY_PATH=target/release at runtime)
cc -o example example.c -L target/release -ltxm_c

# Static linking
cc -o example example.c target/release/libtxm_c.a
```

## License

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))
