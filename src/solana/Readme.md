To build:
make solana

To clean:
make clean

To deploy:
solana program deploy ../../target/program/solana.so


Issue: Writeable Shared/Static Data in nim object files:

in the main c file nim generates:

N_LIB_PRIVATE int cmdCount;
N_LIB_PRIVATE char** cmdLine;
N_LIB_PRIVATE char** gEnv;



removing these 3 seems to cause no issues as far as i'm aware.

In stdlib_system.nim.c nim generates:

N_LIB_PRIVATE tyProc__9axCnCRMUx32AHzFgBrzSMg globalRaiseHook__system_2052;
N_LIB_PRIVATE tyProc__9axCnCRMUx32AHzFgBrzSMg localRaiseHook__system_2055;
N_LIB_PRIVATE tyProc__T4eqaYlFJYZUv9aG9b1TV0bQ outOfMemHook__system_2057;
N_LIB_PRIVATE tyProc__NFmM6mqUOVW3cJg4yvk8Fw unhandledExceptionHook__system_2060;
N_LIB_PRIVATE NIM_BOOL nimInErrorMode__system_2302;
N_LIB_PRIVATE N_NIMCALL(void, panic__system_1954)(NimStringDesc* s) {
}
static N_INLINE(void, sysFatal__system_2263)(NimStringDesc* message) {
    panic__system_1954(message);
}
N_LIB_PRIVATE N_NIMCALL(void, nimTestErrorFlag)(void) {
    {
        if (!nimInErrorMode__system_2302) goto LA3_;
        sysFatal__system_2263(((NimStringDesc*) &TM__Q5wkpxktOdTGvlSRo9bzt9aw_2));
    }
    LA3_: ;
}


which can be replaced with:

N_LIB_PRIVATE N_NIMCALL(void, panic__system_1954)(NimStringDesc* s) {
}
static N_INLINE(void, sysFatal__system_2263)(NimStringDesc* message) {
    panic__system_1954(message);
}
N_LIB_PRIVATE N_NIMCALL(void, nimTestErrorFlag)(void) {
    {

    }
    LA3_: ;
}



running the command nm filename.o | grep -i " [BCDGS] " spits out any writeable static/shared data in the object files
