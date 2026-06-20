(block 0100h
  (lxi H 0400h)
  (call 0200h)
  (hlt))

(block 0200h
  (mov A M)
  (cpi 00h)
  (jz 0300h)
  (out 01h)
  (inx h)
  (jmp 0200h))

(block 0300h
  (ret))

(block 0400h
  (db 'h' 'e' 'l' 'l' 'o' 20h 'w' 'o' 'r' 'l' 'd' 00h))