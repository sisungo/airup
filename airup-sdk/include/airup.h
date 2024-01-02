//! File: `airup.h`: Main header file of Airup SDK for C
//! License: MIT

#pragma once

#include <stdint.h>

#ifndef _AIRUP_H
#define _AIRUP_H

struct airup_error {
    uint32_t code;
    const char *message;
    const void *payload;
};

#ifdef __cplusplus
extern "C" {
#endif /* __cplusplus */

#ifdef __cplusplus
}
#endif /* __cplusplus */

#endif /* AIRUP_H */