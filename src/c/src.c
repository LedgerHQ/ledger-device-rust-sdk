#include "os.h"

void os_longjmp(unsigned int exception) {
  longjmp(try_context_get()->jmp_buf, exception);
}