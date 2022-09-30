#ifndef MODULETOS_PIT_H
#define MODULETOS_PIT_H

#include <stdint.h>

#define PIT_CHANNEL_0_DATA  0x40
#define PIT_CHANNEL_1_DATA  0x41
#define PIT_CHANNEL_2_DATA  0x42
#define PIT_COMMAND         0x43

namespace PIT {
    extern double TimeSinceBoot;
    const uint64_t BaseFrequency = 1193182;

    void Sleepd(double seconds);
    void Sleep(uint64_t milliseconds);

    void SetDivisor(uint16_t divisor);
    void InitPIT();
    uint64_t GetFrequency();
    void SetFrequency(uint64_t frequency);
    void SetInterval(uint64_t interval);
    void Tick();
}

#endif //MODULETOS_PIT_H
