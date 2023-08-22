#include "exceptions.h"
#include "os_lib.h"
#include "cx_errors.h"
#include "os.h"

#define true 1
#define false 0

cx_err_t os_lib_call_no_throw(unsigned int *call_parameters PLENGTH(3*sizeof(unsigned int))) {
    cx_err_t err = CX_OK;
    //mcu_usb_printf("In os_lib_call_no_throw()\n");
    BEGIN_TRY {
        TRY {
            os_lib_call(call_parameters);
        }
        CATCH_OTHER(e) {
            err = CX_INTERNAL_ERROR;
        }
        FINALLY {
        }
    }
    END_TRY;
        return err;
}