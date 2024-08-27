# GOTO Viewer

This project is still in early development stages.

GOTO viewer was originally envisioned as tool to facilitate development, 

## Goal

The goal here is to have a tool that facilitates visualizing and changing GOTO programs generated from ESBMC and CBMC by:

- Parsing the GBF (goto binary format?) from ESBMC and CBMC
- Writing GBF into an db (sqlite). 
- Writing into GBF to ESBMC/CBMC. Allowing to convert between both versions (note that this is not a compatibility layer, I expect a third-party app that will use the sqlite to adapt the program).
- Parsing a db into ESBMC/CBMC GBF.

## Formats

### GBF

### DB

The format is essentially a container of strings references. It is hard to manually change things, but it is pretty trivial for a relational DB.


## Intrinsic functions

ESBMC and CBMC also relies on some intrinsic functions. These need operational models (or equivalent implementations) for the translation to work properly.
Also, some are the same with different names, e.g.: `__CPROVER__start = __ESBMC_main`

### CBMC

```
__CPROVER__start
__CPROVER_architecture_NULL_is_zero
__CPROVER_architecture_arch
__CPROVER_architecture_endianness
__CPROVER_architecture_alignment
__CPROVER_architecture_wchar_t_is_unsigned
__CPROVER_architecture_char_is_unsigned
__CPROVER_initialize
__CPROVER_max_malloc_size
__CPROVER_size_t
__CPROVER_memory_leak
__CPROVER_object_whole
__CPROVER_dead_object
__CPROVER_object_upto
__CPROVER_deallocated
__CPROVER_object_from
__CPROVER_memory
__CPROVER_constant_infinity_uint
__CPROVER_assignable
__CPROVER_architecture_memory_operand_size
__CPROVER_rounding_mode
__CPROVER_architecture_os
__CPROVER_architecture_long_int_width
__CPROVER_freeable
__CPROVER_architecture_int_width
__CPROVER_is_freeable
__CPROVER_architecture_word_size
__CPROVER_was_freed
__CPROVER_architecture_bool_width
__CPROVER_architecture_char_width
__CPROVER_architecture_short_int_width
__CPROVER_architecture_long_long_int_width
__CPROVER_architecture_pointer_width
__CPROVER_architecture_single_width
__CPROVER_architecture_double_width
__CPROVER_architecture_long_double_width
__CPROVER_architecture_wchar_t_width
```

### ESBMC

```
__ESBMC_num_total_threads
__ESBMC_num_threads_running
__ESBMC_unreachable
__ESBMC_alloc_size
__ESBMC_alloc
__ESBMC_same_object
__ESBMC_init_object
__ESBMC_POINTER_OFFSET
__ESBMC_atomic_end
__ESBMC_atexit_handler
__ESBMC_HIDE
__ESBMC_tmp
__ESBMC_atomic_begin
__ESBMC_pthread_end_main_hook
__ESBMC_yield
__ESBMC_pthread_start_main_hook
__ESBMC_get_object_size
__ESBMC_is_dynamic
__ESBMC_POINTER_OBJECT
__ESBMC_assert
__ESBMC_memset
__ESBMC_assume
__ESBMC_rounding_mode
__ESBMC_is_little_endian
__ESBMC_bitcast
__ESBMC_memory_leak_checks
__ESBMC_main
```
