#ifndef _AIRUP_H
#define _AIRUP_H

#ifdef __cplusplus
extern "C" {
#endif /* __cplusplus */

/*
 * Notify Airup daemon for this service to start.
 *
 * If any error would occur, it would be silently passed without report.
 */
void Airup_Notify();

#ifdef __cplusplus
}
#endif /* __cplusplus */

#endif /* _AIRUP_H */
