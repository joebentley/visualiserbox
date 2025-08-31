# visualiserbox

By typing, a "program" consisting of one character operators is written onto the screen. Every frame, for each pixel, the pixel coordinates `x` and `y`, and the time `t` are pushed onto a stack, and the program is executed. If, at any point, the stack is empty when popped, a value is taken from a three element ring buffer containing `[x, y, t]` which is then cycled (`p -> (p + 1) % 3`). The remaining three elements on the stack at the end of the program's execution are used as `(h, s, v)` for that pixel.

https://github.com/user-attachments/assets/558b596f-1308-476d-a9a9-559167360e0d

