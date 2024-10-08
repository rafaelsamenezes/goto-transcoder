## Architecture

```mermaid
flowchart LR
    A[GOTO] --> R
    R[Reader] --> G
    G[Abstract GOTO]
    G --> W
    W[Writer]
    W --> T[GOTO]
```

### Reader

A reader is a type class responsible to parse a GBF (goto binary format) into the Abstract Goto Grammar. This is project dependent and may change without any notice. This is usually fine for reading numbers, strings and references. The issue
raises from the incompatibility of instrumentations. There a few interesting files from both ESBMC/CBMC to extract the implementation: `read_goto_bin.cpp`, `irep_serialization.cpp` and the primitives themselves. That being said, most of the basics from both ESBMC/CBMC should be fine at this point.

### Writer

### Abstract GOTO

A full GOTO program consists in a set of Symbols and a set of Functions. 

#### Irep

Considering ESBMC and CBMC the most important data structure to consider is the Irep (intermediate representation?). It is a string based format that is used to contain all values from the program. As an example for the constant 42:

```
Irep {
  "id": "constant"
  "named_subt":
    "type": "unsigned_bv"
    "value": "42"
  "comment:"
    "location:" "main.c function foo"
}
```

For this project, the spec is:

```
Irep {
  id: String,
  sub: Vec<Irep>,
  named_sub: HashMap<String, Irep>,
  comment_sub: HashMap<String, Irep>
}
```

Neither CBMC not ESBMC will use a "String" directly, they use a string cache which is only used by reference. This is also true for the binary formats.


#### Symbol

A symbol is an Irep of the form:

```
Irep {
 id = "symbol", // required
 named_subt:
   "type": <irep>, //required
   "symvalue": <irep>, // optional
   "location": <irep>, // optional
   "name": <irep>, // required
   "module": <irep>, // required
   "base_name": <irep> // required
   "mode": <irep> // required
   // TODO: flags // required
}
```

For example, an instruction `int a = 42;` in the function `foo` might generate:

```
Irep {
 id = "symbol",
 named_subt:
   "type": Irept { id = "constant_bv", "named_subt"["width"]: "4" }
   "symvalue": {id = "42" }, 
   "location": {id = "foo line 1"},
   "name": {id = "c:foo@a"}, 
   "module": {id = "foo"}, 
   "base_name": {id = "a"}
   "mode": {id = "C" } 
}
```

#### Function

A Function is of the pair <String, Irept>, where the first is the function name and the second is the set of instructions (in Irep):

```
Irep {
  id = "goto-program", // required
  subt: <instructions> // optional
}
```


