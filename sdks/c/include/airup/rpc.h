#pragma once

#ifndef _AIRUP_RPC_H
#define _AIRUP_RPC_H

typedef struct {
    int sockfd;
} airup_conn_t;

#ifdef __cplusplus
extern "C" {
#endif /* __cplusplus */

int airup_connect(airup_conn_t *obj, const char *path);
void airup_disconnect(airup_conn_t obj);

#ifdef __cplusplus
}
#endif /* __cplusplus */

#endif
