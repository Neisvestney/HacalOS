#include "efiRuntimeServices.h"

namespace EFI {
    RuntimeServices *runtimeServices;

    double Timestamp() {
        Time time;
        runtimeServices->GetTime(&time, nullptr);

        double result = ((time.Year - 1970) * 31556926) +
                        (time.Month * 2629743) +
                        (time.Day * 86400) +
                        (time.Hour * 3600) +
                        (time.Minute * 60) +
                        time.Second +
                        (double) time.Nanosecond / 100;

        return result;
    }
}