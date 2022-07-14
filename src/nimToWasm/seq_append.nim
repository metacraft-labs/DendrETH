proc appender*(a, b: int): seq[int] =
  @[a,b]

proc main*(x, y: int): int =
  return appender(x, y).len
