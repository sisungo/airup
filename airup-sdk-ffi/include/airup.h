#pragma once

#include <stddef.h>
#include <stdint.h>

#ifndef _AIRUP_H
#define _AIRUP_H

#define AIRUP_EIO 1

struct AirupSDK_Error {
    uint32_t code;
    char *message;
    void *info;
}

#ifdef __cplusplus
extern "C" {
#endif /* __cplusplus */

struct AirupSDK_Error AirupSDK_GetLastError(void);
void *AirupSDK_OpenConnection(const char *path);
void AirupSDK_CloseConnection(void *conn);
int AirupSDK_SendMessage(void *conn, unsigned char *data, size_t len);
int AirupSDK_RecvMessage(void *conn, unsigned char **data, size_t *len);

#ifdef __cplusplus
}
#endif /* __cplusplus */

#endif /* _AIRUP_H */