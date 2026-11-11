; ATmega328P PB5 blink, bare metal.
; Toggles PORTB bit 5 with a simple software delay.

.equ DDRB, 0x04
.equ PORTB, 0x05
.equ PB5, 5

.org 0x0000
    ldi r16, (1 << PB5)
    out DDRB, r16
loop:
    out PORTB, r16
    ldi r17, 0xFF
delay:
    dec r17
    brne delay
    ldi r18, (1 << PB5)
    eor r16, r18
    rjmp loop
