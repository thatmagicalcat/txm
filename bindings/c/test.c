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
