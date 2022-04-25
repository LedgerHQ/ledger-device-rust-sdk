#pragma once


#include "os_hal.h"

#include <stddef.h>
#include <stdint.h>

#ifdef HAVE_BOLOS

#if defined(ST31)
#include <core_sc000.h>
#endif

#if defined(STM32)
#define __MPU_PRESENT 1
#include <stm32l0xx.h>
#endif

#if defined(ST33J2M0)
#include <core_sc300.h>
#endif

#if defined(ST33K1M5)
#include <core_cm35p.h>
#endif

#if defined(X86)
#define NATIVE_PRINT
#include <setjmp.h>
#include <stdio.h>
#endif

#endif // HAVE_BOLOS

#define WIDE
