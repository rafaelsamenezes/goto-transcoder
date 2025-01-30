 # Development

Most of the development/debugging right now is done by manually running the tool and checking if the output makes sense. This includes:

1. Generating the CBMC gbf (`goto-cc`)
2. Checking the output of CBMC for such a program (`cbmc <gbf file> --show-goto-functions`)
3. Calling goto-transcoder to convert the gbf into esbmc gbf (check README.md for latest instructions)
4. Invoking ESBMC with the converted gbf (`esbmc --binary <converted file> --goto-functions-only`)
5. Verifying the program with ESBMC (`esbmc <converted file> <strategy>`)

Note that ESBMC being able to parse the code does not mean that it was converted correctly. Some issues are very easy
to catch such as number that was supposed to be 4 is now -42, however some of them can be very tricky when the issue is related to types mismatches or invalid parsing.

## Adding test cases

1. Define a goto-cc input or generate an CBMC compatible goto file and add it to `resources/test`. You can also just create a C source file
2. At `adapter.rs` you can add the test cases.

For example:

```rust
#[test]
#[ignore]
fn hello_world() {
  println!("Remember to set GOTO_CC and ESBMC environment variables!");
  // Basic
  run_test("hello_world.c", &["--goto-functions-only"], 0);
  run_test("hello_world.c", &["--incremental-bmc"], 0);
  run_test("hello_world_fail.c", &["--incremental-bmc"], 1);
}
```

The function `run_test(c-file, args, exit-code)` compiles a C code (from `resources/test`) with CBMC, converts to ESBMC, then run it with the `args`. If the exit code from ESBMC matches `exit-code` then the test succeeds.

The `#[ignore]` means that the test will only run by using `cargo test -- --ignored`

## Debugging expressions

The entry point for the CBMC parsing is the function `process_cbmc_file` inside the `cbmc.rs`. It consists in three main steps:
1. Header validation. Failures here means that either an incompatible version of CBMC is being used or that the bytereader has problems. Consulting the `read_bin_goto_binary` files and functions inside cbmc can provide some guidance.
2. Symbol table. All symbols are sequentially parsed into irep expressions, ireps can be printed at any time with a common format `println!("My irep: {}", irep)`
3. Function parsing. Each function also contains a set of instructions that is sequentially parsed and associated into the function. Similarly, each function can be printed using the fmt.

### Fixing CBMC expressions

In case the direct convertion results in a crash or incorrect parameter, it may be because ESBMC is eexpecting the IREP in a 
different form. The easiest way to debug this is to check `migrate.cpp` of ESBMC and check which parameters are expected.

As an example, one fix that it is needed for expressions is that in CBMC an expression such as 1 + 2 is defined (similarly):

```
id: "+"
  - [constant 1, constant 2]
  - type: ...
  ...
```

ESBMC however expect this expression to be of the following form:

```
id: "+"
  - operands:
    - [constant 1, constant 2]
  - type: ...
  ...
```

We can know that by checking this specific case `migrate_expr`:

```cpp
  else if (expr.id() == exprt::plus)
  {
    type = migrate_type(expr.type());

    expr2tc side1, side2;
    if (expr.operands().size() > 2)
    {
      splice_expr(expr, new_expr_ref);
      return;
    }

    convert_operand_pair(expr, side1, side2);

    new_expr_ref = add2tc(type, side1, side2);
  }

```

In here, `exprt::plus` is "+" (I highly recommend setting an LSP to ESBMC and CBMC) while `.operands()` access the "operands" field. All these hacks are defined in the function `esbmcfixes::fix_expression` at `adapter.rs`. Adding new expressions is low hanging fruit (see list of equivalences).


# List of Irep equivalences

Here is the tracking of the ireps that were already implemented and have at least two tests.

## Types

| ESBMC      | CBMC     | Implemented |
|------------|----------|-------------|
| signedbv   | signedbv | Y           |
| pointer    | pointer  | Y           |
| unsignedbv | ?        | N           |
| complex    | ?        | N           |
| floatbv    | ?        | N           |
| fixedbv    | ?        | N           |
| bool       | bool     | Y           |
| empty      | ?        | Y           |
| symbol     | ?        | N           |
| struct     | ?        | Y           |
| union      | ?        | N           |
| class      | ?        | N           |
| code       | ?        | N           |
| array      | array    | Y           |
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
| mod                | mod         | Y           |
| =                  | =           | Y           |
| notequal           | notequal    | Y           |
| index              | index       | N           |
| array-of           | ?           | N           |
| object-descriptor  | ?           | N           |
| dynamic-object     | ?           | N           |
| typecast           | typecast    | Y           |
| =>                 | ?           | N           |
| and                | ?           | Y           |
| xor                | ?           | N           |
| or                 | ?           | Y           |
| not                | ?           | Y           |
| address-of         | address-of  | Y           |
| dereference        | dereference | Y           |
| if                 | ?           | Y           |
| with               | ?           | N           |
| member             | ?           | N           |
| isnan              | ?           | N           |
| ieee-float-equal   | ?           | N           |
| type               | ?           | N           |
| constant           | constant    | N           |
| true               | true        | Y           |
| false              | false       | Y           |
| <                  | <           | Y           |
| >                  | >           | Y           |
| <=                 | <=          | N           |
| >=                 | >=          | N           |
| bitand             | bitand      | N           |
| bitor              | bitor       | N           |
| bitxor             | bitxor      | N           |
| bitnand            | bitnand     | N           |
| bitnor             | bitnor      | N           |
| bitnxor            | bitnxor     | N           |
| bitnot             | bitnot      | N           |
| ashr               | ashr        | Y           |
| lshr               | lshr        | Y           |
| shl                | shl         | Y           |
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

### Unsupported stuff

- Quantifiers
