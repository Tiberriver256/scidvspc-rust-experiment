#!/bin/bash
cd /root/repos/scidvspc-code
hexdump -C bases/matein1.sg4 -s 739105 -n 100 | head -10
