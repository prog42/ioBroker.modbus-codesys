#!/bin/bash

SYMBOL_XML_FILE=$(ls -Art *.SYM_XML 2> /dev/null | tail -n 1)

if [ -z "$SYMBOL_XML_FILE" ]; then
  SYMBOL_XML_FILE=$(ls -Art ../*.SYM_XML 2> /dev/null | tail -n 1)
fi

if [ -z "$SYMBOL_XML_FILE" ]; then
  echo "could not find symbol-xml"
  exit
fi

BASE=$(basename $SYMBOL_XML_FILE)

ioBroker-modbus-codesys-convert -S $SYMBOL_XML_FILE -F iob

cp inputs-out.csv $BASE-inputs-out.csv
cp holdings-out.csv $BASE-holdings-out.csv
