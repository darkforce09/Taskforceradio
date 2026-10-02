/*
 * The stand-in for FAudio's src/FAudio_internal.h that the vendored oracle/F3DAudio.c includes.
 *
 * The real header pulls in FAudio.h, FAPOBase.h and SDL; F3DAudio.c needs only the few macros
 * below, so they map straight onto the C library. FAudio_assert does not abort: it reports to
 * oracle_shim_record_assertion (shim/oracle_shim.cpp), which counts the failed parameter checks of
 * the current call per thread, so a test can see which X3DAudio rules a scene breaks.
 */

#ifndef TFAR_GOLDEN_VECTORS_FAUDIO_INTERNAL_H
#define TFAR_GOLDEN_VECTORS_FAUDIO_INTERNAL_H

#include <math.h>
#include <stddef.h>
#include <stdint.h>
#include <string.h>

#ifdef __cplusplus
extern "C" {
#endif

/* Counts one failed parameter check of the current F3DAudio call when holds is 0. */
void oracle_shim_record_assertion(int holds);

#ifdef __cplusplus
}
#endif

#define FAudio_assert(condition) oracle_shim_record_assertion((condition) ? 1 : 0)

#define FAudio_zero(ptr, size) memset(ptr, '\0', size)

#define FAudio_cosf(x) cosf(x)
#define FAudio_sinf(x) sinf(x)
#define FAudio_sqrtf(x) sqrtf(x)
#define FAudio_acosf(x) acosf(x)
#define FAudio_atan2f(y, x) atan2f(y, x)
#define FAudio_fabsf(x) fabsf(x)

#define FAudio_min(val1, val2) (val1 < val2 ? val1 : val2)
#define FAudio_max(val1, val2) (val1 > val2 ? val1 : val2)
#define FAudio_clamp(val, min, max) (val > max ? max : (val < min ? min : val))

#define ALIGN(type, boundary) type __attribute__((aligned(boundary)))

/* FAudio.h's error code for a bad call, returned by F3DAudioInitialize8. */
#define FAUDIO_E_INVALID_CALL 0x88960001

#endif /* TFAR_GOLDEN_VECTORS_FAUDIO_INTERNAL_H */
