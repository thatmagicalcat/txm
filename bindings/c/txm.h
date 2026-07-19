#ifndef TXM_H
#define TXM_H

#ifdef __cplusplus
extern "C" {
#endif

#include <stddef.h>

char *txm_render(const char *input);

void txm_free_string(char *s);

#ifdef __cplusplus
}
#endif

#endif /* TXM_H */
