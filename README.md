# GOTO Viewer

This project is still in early development stages.

GOTO viewer was originally envisioned as tool to facilitate development, 

## Goal

The goal here is to have a tool that facilitates visualizing and changing GOTO programs generated from ESBMC and CBMC by:

- Parsing the GBF (goto binary format?) from ESBMC and CBMC
- Writing GBF into SQLite. GBF format is essentially a container of strings references. It is hard to manually change things, but it is pretty trivial for a relational DB.
- Writing into GBF for CBMC and ESBMC
- Writing SQLITE3 into ESBMC/CBMC GBF.
