# proc printf(frmt: cstring) {.varargs, importc, header: "<stdio.h>", cdecl.}
# proc exit(code: int) {.importc, header: "<stdlib.h>", cdecl.}

{.push stack_trace: off, profiler:off.}

proc rawoutput(s: string) =
  discard
#   printf("%s\n", s)

proc panic(s: string) {.noreturn.} =
  discard
#   rawoutput(s)
#   exit(1)

{.pop.}