#ifndef LOGGING_H
#define LOGGING_H

#include "../rss/include/acc_definitions_common.h"

void c_log_stub(acc_log_level_t level, const char *module, const char *format,
                ...);

#endif // LOGGING_H
