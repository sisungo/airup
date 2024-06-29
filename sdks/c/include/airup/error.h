#pragma once

#ifndef _AIRUP_ERROR_H
#define _AIRUP_ERROR_H

#define AIRUP_NO_ERROR 0
#define AIRUP_IO_ERROR 32

union airup_error_payload {
    int sys_errno;
};

struct airup_error {
    int code;
    union airup_error_payload payload;
};

#ifdef __cplusplus
extern "C" {
#endif /* __cplusplus */

void airup_set_error(struct airup_error error);
struct airup_error airup_get_error(void);

#ifdef __cplusplus
}
#endif /* __cplusplus */

#endif
