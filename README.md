# GOTO Viewer

This project is still in early development stages.

## Goal

The goal here is to have a tool that facilitates visualizing and manually altering GOTO programs generated from ESBMC and CBMC by:

- Parsing the GBF (goto binary format?) from ESBMC and CBMC
- Writing GBF into SQLITE3. The format is essentially a container of strings references. It is hard to manually change things, but it is pretty trivial for a relational DB.
- Writing into GBF to ESBMC/CBMC. Allowing to convert between both versions (note that this is not a compatibility layer, I expect a third-party app that will use the sqlite to adapt the program).
- Writing SQLITE3 into ESBMC/CBMC GBF.


