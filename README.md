# GOTO Transcoder

This project is still in early development stages. The goal here is to have a tool that facilitates visualizing and changing GOTO programs generated from ESBMC and CBMC by:
- Parsing the GBF (goto binary format?) from ESBMC and CBMC
- Writing into GBF to ESBMC/CBMC. Allowing to convert between both versions.
 
### Use

1. Generate the GBF from CBMC: `goto-cc file.c`. This will generate an `a.out`, for this example we will rename it to `file-cbmc.goto`.
2. To convert from CBMC into ESBMC: `cargo run -- --mode 0 --input file-cbmc.goto --output file-esbmc.goto`.
3. Run ESBMC: `esbmc --binary file-esbmc.goto --goto-functions-only`.

### Contributing

- [Architecture](docs/Architecture.md)
- [Development](docs/Development.md)
