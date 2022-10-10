# A minimal `panicoverride.nim` file necessary for compiling in standalone mode

{.push stack_trace: off, profiler:off.}

proc rawoutput(s: string) =
  discard

proc panic(s: string) {.noreturn.} =
  discard

{.pop.}
