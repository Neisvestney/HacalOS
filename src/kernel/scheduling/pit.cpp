#include "pit.h"
#include "../io.h"

namespace PIT{
    double TimeSinceBoot = 0;

    uint16_t Divisor = 65535;

    void Sleepd(double seconds){
        double startTime = TimeSinceBoot;
        while (TimeSinceBoot < startTime + seconds){
            asm("hlt");
        }
    }

    void Sleep(uint64_t milliseconds){
        Sleepd((double)milliseconds / 1000);
    }

    void SetDivisor(uint16_t divisor){
        if (divisor < 100) divisor = 100;
        Divisor = divisor;
        outb(PIT_CHANNEL_0_DATA, (uint8_t)(divisor & 0x00ff));
        io_wait();
        outb(PIT_CHANNEL_0_DATA, (uint8_t)((divisor & 0xff00) >> 8));
    }

    void InitPIT() {
        /* name | value | size | desc
        * --------------------------
        * chan |     0 |    2 | the channel to use, channel 0 = IRQ0
        * acs  |   0x3 |    2 | how the divider is sent, 3 = lobyte then hibyte
        * mode |   0x3 |    3 | the mode of the pit, mode 3 = square wave
        * bcd  |     0 |    1 | bcd or binary mode, 0 = binary, 1 = bcd
        */
        uint8_t data = (1 << 5) | (1 << 4) | (1 << 2) | (1 << 1) | 0x00;
        outb(PIT_COMMAND, data);
    }

    uint64_t GetFrequency(){
        return BaseFrequency / Divisor;
    }

    void SetFrequency(uint64_t frequency){
        SetDivisor(BaseFrequency / frequency);
    }

    void SetInterval(uint64_t interval)
    {
        uint64_t frequency = 1000 / interval;
        uint16_t divider = (uint16_t) (BaseFrequency / frequency);
        SetDivisor(divider);
    }

    void Tick(){
        TimeSinceBoot += 1 / (double)GetFrequency();
    }
}
