; ATmega328P Timer0 overflow interrupt demo.
; Counts overflows and writes a marker to r17 in the ISR.

.equ TCCR0A, 0x24
.equ TCCR0B, 0x25
.equ TIMSK0, 0x6E

.org 0x0000
    rjmp start
.org 0x0020
    ldi r17, 7
    reti

start:
    ldi r16, 1
    out TCCR0A, r16
    ldi r16, 1
    out TIMSK0, r16
    ldi r18, 255
delay:
    dec r18
    brne delay
    rjmp start
