# Development

## Debugging expressions

### Debugging unsuported CBMC expressions
### Debugging unsupported ESBMC expressions

## Dealing with unsupported stuff

# List of Irep equivalences

Here is the tracking of the ireps that were already implemented and have at least two tests.

## Types

| ESBMC      | CBMC     | Implemented |
|------------|----------|-------------|
| signedbv   | signedbv | y           |
| pointer    | pointer  | Y           |
| unsignedbv | ?        | N           |
| complex    | ?        | N           |
| floatbv    | ?        | N           |
| fixedbv    | ?        | N           |
| bool       | ?        | N           |
| empty      | ?        | N           |
| symbol     | ?        | N           |
| struct     | ?        | N           |
| union      | ?        | N           |
| class      | ?        | N           |
| code       | ?        | N           |
| array      | ?        | N           |
| #reference | ?        | N           |
| bv         | ?        | N           |
| vector     | vector   | N           |
| intcap     | N/A      | N           |
| uintcap    | N/A      | N           |


## Expressions

| ESBMC              | CBMC        | Implemented |
|--------------------|-------------|-------------|
| trans              | ?           | N           |
| symbol             | ?           | N           |
| +                  | +           | Y           |
| -                  | -           | Y           |
| *                  | *           | Y           |
| /                  | /           | Y           |
| mod                | mod         | N           |
| =                  | =           | Y           |
| notequal           | notequal    | Y           |
| index              | index       | N           |
| array-of           | ?           | N           |
| object-descriptor  | ?           | N           |
| dynamic-object     | ?           | N           |
| typecast           | typecast    | Y           |
| =>                 | ?           | N           |
| and                | ?           | N           |
| xor                | ?           | N           |
| or                 | ?           | N           |
| not                | ?           | N           |
| address-of         | address-of  | Y           |
| dereference        | dereference | Y           |
| if                 | ?           | N           |
| with               | ?           | N           |
| member             | ?           | N           |
| isnan              | ?           | N           |
| ieee-float-equal   | ?           | N           |
| type               | ?           | N           |
| constant           | constant    | N           |
| true               | true        | Y           |
| false              | false       | Y           |
| <                  | <           | Y           |
| >                  | >           | N           |
| <=                 | <=          | N           |
| >=                 | >=          | N           |
| bitand             | bitand      | N           |
| bitor              | bitor       | N           |
| bitxor             | bitxor      | N           |
| bitnand            | bitnand     | N           |
| bitnor             | bitnor      | N           |
| bitnxor            | bitnxor     | N           |
| bitnot             | bitnot      | N           |
| ashr               | ashr        | N           |
| lshr               | lshr        | N           |
| shl                | shl         | N           |
| abs                | abs         | N           |
| argument           | ?           | N           |
| sideffect          | ?           | N           |
| code               | ?           | N           |
| skip               | ?           | N           |
| assign             | ?           | N           |
| bitcast            | ?           | N           |
| nearbyint          | ?           | N           |
| abs                | ?           | N           |
| ieee-add           | ?           | N           |
| ieee-sub           | ?           | N           |
| ieee-mul           | ?           | N           |
| ieee-div           | ?           | N           |
| ieee-fma           | ?           | N           |
| ieee-sqrt          | ?           | N           |
| popcount           | ?           | N           |
| bswap              | ?           | N           |
| same-object        | ?           | N           |
| pointer-offset     | ?           | N           |
| pointer-object     | ?           | N           |
| pointer-capability | N/A         | N           |
| byte-extract       | ?           | N           |
| byte-update        | ?           | N           |
| code-block         | ?           | N           |
| code-assign        | ?           | N           |
| code-init          | ?           | N           |
| code-decl          | ?           | N           |
| code-dead          | ?           | N           |
| code-printf        | ?           | N           |
| code-expression    | ?           | N           |
| code-return        | ?           | N           |
| code-skip          | ?           | N           |
| code-free          | ?           | N           |
| code-goto          | ?           | N           |
| code-function-call | ?           | N           |
| code-comma         | ?           | N           |
| invalid-pointer    | ?           | N           |
| code-asm           | ?           | N           |
| isinf              | ?           | N           |
| isnormal           | ?           | N           |
| isfinite           | ?           | N           |
| signbit            | ?           | N           |
| concat             | ?           | N           |
| extract            | ?           | N           |




### Intrinsic Functions

ESBMC and CBMC also relies on some intrinsic functions. These need operational models (or equivalent implementations) for the translation to work properly.
Also, some are the same with different names, e.g.: `__CPROVER__start = __ESBMC_main`

#### CBMC 

```
__CPROVER__start
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
__CPROVER_freeable
__CPROVER_is_freeable
__CPROVER_was_freed


__CPROVER_rounding_mode
__CPROVER_architecture_memory_operand_size
__CPROVER_architecture_os
__CPROVER_architecture_long_int_width
__CPROVER_architecture_int_width
__CPROVER_architecture_word_size
__CPROVER_architecture_NULL_is_zero
__CPROVER_architecture_arch
__CPROVER_architecture_endianness
__CPROVER_architecture_alignment
__CPROVER_architecture_wchar_t_is_unsigned
__CPROVER_architecture_char_is_unsigned
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

#### ESBMC

```
_Bool __ESBMC_is_allocated[&symbol] __infinity
_Bool __ESBMC_is_free[&symbol] __infinity
size_t __ESBMC_allocated_size[&symbol] __infinity

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