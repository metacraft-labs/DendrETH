typedef __SIZE_TYPE__ size_t;
// typedef __PTRDIFF_TYPE__ ptrdiff_t;

 #ifdef __cplusplus
 #  if !defined(__MINGW32__) && !defined(_MSC_VER)
 #    define NULL __null
 #  else
 #    define NULL 0
 #  endif
 #else
 #  define NULL ((void*)0)
 #endif

 #ifdef __cplusplus
 #if defined(_MSC_EXTENSIONS) && defined(_NATIVE_NULLPTR_SUPPORTED)
 namespace std { typedef decltype(nullptr) nullptr_t; }
 using ::std::nullptr_t;
 #endif
 #endif
