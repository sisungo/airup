#include <airup/error.h>

static _Thread_local struct airup_error airup_error = { AIRUP_NO_ERROR };

void airup_set_error(struct airup_error error) {
    airup_error = error;
}

struct airup_error airup_get_error(void) {
    return airup_error;
}
