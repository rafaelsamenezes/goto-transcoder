# GOTO Viewer

This project is still in early development stages.

GOTO viewer was originally envisioned as tool to facilitate development, 

## Goal

The goal here is to have a tool that facilitates visualizing and changing GOTO programs generated from ESBMC and CBMC by:

- Parsing the GBF (goto binary format?) from ESBMC and CBMC
- Writing GBF into an db (sqlite). 
- Writing into GBF to ESBMC/CBMC. Allowing to convert between both versions (note that this is not a compatibility layer, I expect a third-party app that will use the sqlite to adapt the program).
- Parsing a db into ESBMC/CBMC GBF.


## GBF

## DB

The format is essentially a container of strings references. It is hard to manually change things, but it is pretty trivial for a relational DB.

