# ioBroker.modbus-codesys
Converter of Codesys symbol file to ioBroker.modbus CSV

## create Debian package
Install cargo-deb
```sh
cargo install cargo-deb
```
Build Debian package
```sh
cargo deb
```
## export symbols from Codesys project
Projekt > Optionen

![export symbols from Codesys project](/doc/codesys-export-symbols.png?raw=true "export symbols from Codesys project")


## run test
```sh
cargo build
cd test/
./ioBroker-modbus-codesys-test.sh
```
The output should look like this:
```
inputs 256: iob_read.a_rd_BYTE
inputs 257: iob_read.b_rd_WORD
inputs 258: iob_read.c_rd_DWORD
inputs 260: iob_read.d_rd_SINT
inputs 261: iob_read.e_rd_INT
inputs 262: iob_read.f_rd_UINT
inputs 264: iob_read.g_rd_DINT
inputs 266: iob_read.h_rd_UDINT
inputs 268: iob_read.i_rd_REAL
inputs 270: iob_read.j_rd_STRING
inputs 312: iob_read.k_rd_TIME
inputs 314: iob_read.l_rd_TOD
inputs 316: iob_read.m_rd_DATE
inputs 318: iob_read.n_rd_DT
inputs 320: iob_read.o_rd_ARRAY_INT.0
inputs 321: iob_read.o_rd_ARRAY_INT.1
inputs 322: iob_read.o_rd_ARRAY_INT.2
inputs 323: iob_read.o_rd_ARRAY_INT.3
inputs 324: iob_read.o_rd_ARRAY_INT.4
holdings 12416: iob_write.a_wr_BYTE
holdings 12417: iob_write.b_wr_WORD
holdings 12418: iob_write.c_wr_DWORD
holdings 12420: iob_write.d_wr_SINT
holdings 12421: iob_write.e_wr_INT
holdings 12422: iob_write.f_wr_UINT
holdings 12424: iob_write.g_wr_DINT
holdings 12426: iob_write.h_wr_UDINT
holdings 12428: iob_write.i_wr_REAL
holdings 12430: iob_write.j_wr_STRING
holdings 12472: iob_write.k_wr_TIME
holdings 12474: iob_write.l_wr_TOD
holdings 12476: iob_write.m_wr_DATE
holdings 12478: iob_write.n_wr_DT
holdings 12480: iob_write.o_wr_ARRAY_INT.0
holdings 12481: iob_write.o_wr_ARRAY_INT.1
holdings 12482: iob_write.o_wr_ARRAY_INT.2
holdings 12483: iob_write.o_wr_ARRAY_INT.3
holdings 12484: iob_write.o_wr_ARRAY_INT.4
```
