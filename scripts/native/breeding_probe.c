/* Read-only exact-ROM native fixture runner. No battery, cheat or ROM-write API.
 * Synthetic RAM and bounded complete native functions only. */
#include <mgba/core/core.h>
#include <mgba/core/log.h>
#include <mgba/internal/arm/arm.h>
#include <mgba/internal/arm/isa-inlines.h>
#include <stdio.h>
static struct mCore *core;
static color_t pixels[240*160];
static void quiet(struct mLogger *log,int category,enum mLogLevel level,const char *format,va_list args) {
    (void)log;(void)category;(void)level;(void)format;(void)args;
}
static struct mLogger logger={.log=quiet};
int start(const char *path) {
    mLogSetDefaultLogger(&logger);
    core=mCoreFind(path);
    if(!core || !core->init(core))return 0;
    mCoreInitConfig(core,"gen3-breeding-probe");
    core->setVideoBuffer(core,pixels,240);
    if(!mCoreLoadFile(core,path))return 0;
    core->reset(core);
    for(unsigned a=0x02000000;a<0x02040000;a++)core->busWrite8(core,a,0);
    for(unsigned a=0x03000000;a<0x03008000;a++)core->busWrite8(core,a,0);
    return 1;
}
void finish(void) { if(core){core->deinit(core);core=NULL;} }
void resetfixture(void) { if(core)core->reset(core); }
void readbytes(unsigned address,unsigned char *bytes,unsigned length) {
    for(unsigned i=0;i<length;i++)bytes[i]=core->busRead8(core,address+i);
}
int writebytes(unsigned address,const unsigned char *bytes,unsigned length) {
    if(!((address>=0x02000000 && address<=0x02040000 && length<=0x02040000-address) || (address>=0x03000000 && address<=0x03008000 && length<=0x03008000-address)))return 0;
    for(unsigned i=0;i<length;i++)core->busWrite8(core,address+i,bytes[i]);
    return 1;
}
int callchoice(unsigned address,unsigned r0,unsigned r1,unsigned r2,unsigned r3,unsigned stop,unsigned second,unsigned *output) {
    struct ARMCore *cpu=core->cpu,saved=*cpu;
    _ARMSetMode(cpu,MODE_THUMB);
    cpu->cpsr.i=1;
    for(int i=0;i<13;i++)cpu->gprs[i]=0;
    cpu->gprs[0]=r0;cpu->gprs[1]=r1;cpu->gprs[2]=r2;cpu->gprs[3]=r3;
    cpu->gprs[13]=0x03007e00;cpu->gprs[14]=0x03007f01;cpu->gprs[15]=address;
    ThumbWritePC(cpu);
    unsigned steps=0;
    unsigned target=stop ? stop+2 : 0x03007f02;
    unsigned other=second ? second+2 : 0xffffffff;
    while(cpu->gprs[15]!=target && cpu->gprs[15]!=other && cpu->gprs[15]!=0x03007f02 && steps++<1000000)core->step(core);
    int ok=cpu->gprs[15]==target ? 1 : cpu->gprs[15]==other ? 2 : 0;
    for(unsigned i=0;i<16;i++)output[i]=cpu->gprs[i];
    /* The board retains active-region and timing state. Keep those coherent
     * when restoring registers after an observation inside ROM (no return BX).
     * Restoring stale CPU memory caches would fetch the next call from BIOS. */
    struct ARMMemory memory=cpu->memory;
    int32_t cycles=cpu->cycles,nextEvent=cpu->nextEvent;
    *cpu=saved;
    cpu->memory=memory;cpu->cycles=cycles;cpu->nextEvent=nextEvent;
    return ok;
}
int calluntil(unsigned address,unsigned r0,unsigned r1,unsigned r2,unsigned r3,unsigned stop,unsigned *output) {
    return callchoice(address,r0,r1,r2,r3,stop,0,output)==1;
}
int callfunc(unsigned address,unsigned r0,unsigned r1,unsigned r2,unsigned r3,unsigned *output) {
    unsigned regs[16];
    int ok=calluntil(address,r0,r1,r2,r3,0,regs);
    *output=regs[0];
    return ok;
}
int allrolls(unsigned address,unsigned stop,unsigned *output) {
    for(unsigned value=0;value<65536;value++) {
        unsigned regs[16];
        if(!calluntil(address,value,0,0,0,stop,regs))return 0;
        output[value]=regs[0];
    }
    return 1;
}
