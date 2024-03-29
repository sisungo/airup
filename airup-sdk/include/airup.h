//! File: airup.h
//! Description: Main header file of the Airup SDK for C
//! License: MIT
//!
//! For documents of usage and API reference of the SDK, see Airup documents.

#pragma once

#include <stdint.h>
#include <stddef.h>

#ifndef _AIRUP_H
#define _AIRUP_H

#define AIRUP_EIO 16
#define AIRUP_EAPI 32
#define AIRUP_EBUFTOOSMALL 64

struct airup_error {
    uint32_t code;
    const char *message;
    const void *payload;
};

struct airup_api_error {
    const char *code;
    const char *message;
    const char *json;
};

typedef struct airup_connection {} airup_connection;

#ifdef __cplusplus
extern "C" {
#endif /* __cplusplus */

struct airup_error airup_last_error(void);
airup_connection *airup_connect(const char *path);
void airup_disconnect(airup_connection *connection);
const char *airup_default_path(void);
const char *airup_build_manifest(void);
int airup_start_service(airup_connection *connection, const char *name);
int airup_stop_service(airup_connection *connection, const char *name);
int airup_trigger_event(airup_connection *connection, const char *id, const char *payload);
int airup_server_version(airup_connection *connection, char *buffer, size_t size);

#ifdef __cplusplus
}
#endif /* __cplusplus */

#endif /* AIRUP_H */
