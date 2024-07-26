.bss
.align 2
_m6global03a03a1A:
    .space 4194304
.align 2
_m6global03a03a1B:
    .space 4194304
.align 2
_m6global03a03a1C:
    .space 4194304
.align 2
_m6global03a03a4main03a03a12ans_4_reduce:
    .space 16
.globl main
.text

main:
addi sp, sp, -64
sw ra, 12(sp)
sw x8, 16(sp)
sw x9, 20(sp)
sw x18, 24(sp)
sw x19, 28(sp)
sw x20, 32(sp)
sw x21, 36(sp)
sw x22, 40(sp)
sw x23, 44(sp)
sw x24, 48(sp)
sw x25, 52(sp)
sw x26, 56(sp)
sw x27, 60(sp)
call getint
mv x22, x10
mv x9, zero
.L1:
bge x9, x22, .L6
slli x19, x9, 12
mv x8, zero
.L3:
bge x8, x22, .L5
slli x5, x8, 2
la ra, _m6global03a03a1A
add ra, ra, x5
add x18, ra, x19
addi x8, x8, 1
call getint
sw x10, 0(x18)
j .L3
.L5:
addi x9, x9, 1
j .L1
.L6:
la x21, _m6global03a03a1B
mv x8, zero
.L7:
bge x8, x22, .L12
slli x19, x8, 12
mv x9, zero
.L9:
bge x9, x22, .L11
slli ra, x9, 2
add ra, x21, ra
add x18, ra, x19
addi x9, x9, 1
call getint
sw x10, 0(x18)
j .L9
.L11:
addi x8, x8, 1
j .L7
.L12:
la x20, _m6global03a03a1C
li ra, 4
div x19, x22, ra
add x18, x19, x19
slli ra, x19, 1
sw ra, 4(sp)
lw ra, 4(sp)
add x9, ra, x19
li ra, 3
mul ra, x19, ra
sw ra, 8(sp)
li x10, 65
call _sysy_starttime
mv ra, zero
sw ra, 0(sp)
.L13:
li x5, 5
lw ra, 0(sp)
bge ra, x5, .L291
li ra, 100
blt ra, x22, .L259
mv x8, zero
.L16:
bge x8, x22, .L42
slli x23, x8, 12
li ra, 1000
blt ra, x22, .L22
mv ra, zero
.L19:
bge ra, x22, .L21
slli x5, ra, 2
add x5, x20, x5
add x5, x5, x23
addi ra, ra, 1
sw zero, 0(x5)
j .L19
.L21:
addi x8, x8, 1
j .L16
.L22:
li x10, 4
call __create_threads
beq x10, zero, .L38
li ra, 1
beq x10, ra, .L34
li ra, 2
beq x10, ra, .L30
lw ra, 8(sp)
.L26:
bge ra, x22, .L28
slli x5, ra, 2
add x5, x20, x5
add x5, x5, x23
addi ra, ra, 1
sw zero, 0(x5)
j .L26
.L28:
li x10, 3
li x11, 4
call __join_threads
.L29:
j .L21
.L30:
lw ra, 4(sp)
.L31:
bge ra, x9, .L33
slli x5, ra, 2
add x5, x20, x5
add x5, x5, x23
addi ra, ra, 1
sw zero, 0(x5)
j .L31
.L33:
li x10, 2
li x11, 4
call __join_threads
j .L21
.L34:
mv ra, x19
.L35:
bge ra, x18, .L37
slli x5, ra, 2
add x5, x20, x5
add x5, x5, x23
addi ra, ra, 1
sw zero, 0(x5)
j .L35
.L37:
li x10, 1
li x11, 4
call __join_threads
j .L21
.L38:
mv ra, zero
.L39:
bge ra, x19, .L41
slli x5, ra, 2
add x5, x20, x5
add x5, x5, x23
addi ra, ra, 1
sw zero, 0(x5)
j .L39
.L41:
mv x10, zero
li x11, 4
call __join_threads
j .L21
.L42:
mv x8, zero
.L43:
bge x8, x22, .L120
slli x5, x8, 2
la ra, _m6global03a03a1A
add x24, ra, x5
slli x23, x8, 12
li ra, 100
blt ra, x22, .L76
mv x25, zero
.L46:
bge x25, x22, .L75
slli x27, x25, 12
add ra, x24, x27
lw x26, 0(ra)
beq x26, zero, .L53
li ra, 1000
blt ra, x22, .L54
mv ra, zero
.L50:
bge ra, x22, .L53
slli x6, ra, 2
add x5, x20, x6
add x7, x5, x27
add x5, x21, x6
add x5, x5, x23
addi ra, ra, 1
lw x5, 0(x5)
mul x6, x26, x5
lw x5, 0(x7)
add x5, x5, x6
sw x5, 0(x7)
j .L50
.L52:
.L53:
addi x25, x25, 1
j .L46
.L54:
li x10, 4
call __create_threads
beq x10, zero, .L70
li ra, 1
beq x10, ra, .L66
li ra, 2
beq x10, ra, .L62
lw ra, 8(sp)
.L58:
bge ra, x22, .L60
slli x6, ra, 2
add x5, x20, x6
add x7, x5, x27
add x5, x21, x6
add x5, x5, x23
addi ra, ra, 1
lw x5, 0(x5)
mul x6, x26, x5
lw x5, 0(x7)
add x5, x5, x6
sw x5, 0(x7)
j .L58
.L60:
li x10, 3
li x11, 4
call __join_threads
.L61:
j .L53
.L62:
lw ra, 4(sp)
.L63:
bge ra, x9, .L65
slli x6, ra, 2
add x5, x20, x6
add x7, x5, x27
add x5, x21, x6
add x5, x5, x23
addi ra, ra, 1
lw x5, 0(x5)
mul x6, x26, x5
lw x5, 0(x7)
add x5, x5, x6
sw x5, 0(x7)
j .L63
.L65:
li x10, 2
li x11, 4
call __join_threads
j .L53
.L66:
mv ra, x19
.L67:
bge ra, x18, .L69
slli x6, ra, 2
add x5, x20, x6
add x7, x5, x27
add x5, x21, x6
add x5, x5, x23
addi ra, ra, 1
lw x5, 0(x5)
mul x6, x26, x5
lw x5, 0(x7)
add x5, x5, x6
sw x5, 0(x7)
j .L67
.L69:
li x10, 1
li x11, 4
call __join_threads
j .L53
.L70:
mv ra, zero
.L71:
bge ra, x19, .L73
slli x6, ra, 2
add x5, x20, x6
add x7, x5, x27
add x5, x21, x6
add x5, x5, x23
addi ra, ra, 1
lw x5, 0(x5)
mul x6, x26, x5
lw x5, 0(x7)
add x5, x5, x6
sw x5, 0(x7)
j .L71
.L73:
mv x10, zero
li x11, 4
call __join_threads
j .L53
.L74:
j .L53
.L75:
addi x8, x8, 1
j .L43
.L76:
li x10, 4
call __create_threads
beq x10, zero, .L110
li ra, 1
beq x10, ra, .L100
li ra, 2
beq x10, ra, .L90
lw x5, 8(sp)
.L80:
bge x5, x22, .L88
slli x12, x5, 12
add ra, x24, x12
lw x11, 0(ra)
beq x11, zero, .L86
mv ra, zero
.L83:
bge ra, x22, .L86
slli x7, ra, 2
add x6, x20, x7
add x10, x6, x12
add x6, x21, x7
add x6, x6, x23
addi ra, ra, 1
lw x6, 0(x6)
mul x7, x11, x6
lw x6, 0(x10)
add x6, x6, x7
sw x6, 0(x10)
j .L83
.L85:
.L86:
addi x5, x5, 1
j .L80
.L87:
j .L86
.L88:
li x10, 3
li x11, 4
call __join_threads
.L89:
j .L75
.L90:
lw ra, 4(sp)
.L91:
bge ra, x9, .L99
slli x12, ra, 12
add x5, x24, x12
lw x11, 0(x5)
beq x11, zero, .L97
mv x5, zero
.L94:
bge x5, x22, .L97
slli x7, x5, 2
add x6, x20, x7
add x10, x6, x12
add x6, x21, x7
add x6, x6, x23
addi x5, x5, 1
lw x6, 0(x6)
mul x7, x11, x6
lw x6, 0(x10)
add x6, x6, x7
sw x6, 0(x10)
j .L94
.L96:
.L97:
addi ra, ra, 1
j .L91
.L98:
j .L97
.L99:
li x10, 2
li x11, 4
call __join_threads
j .L75
.L100:
mv x5, x19
.L101:
bge x5, x18, .L109
slli x12, x5, 12
add ra, x24, x12
lw x11, 0(ra)
beq x11, zero, .L107
mv ra, zero
.L104:
bge ra, x22, .L107
slli x7, ra, 2
add x6, x20, x7
add x10, x6, x12
add x6, x21, x7
add x6, x6, x23
addi ra, ra, 1
lw x6, 0(x6)
mul x7, x11, x6
lw x6, 0(x10)
add x6, x6, x7
sw x6, 0(x10)
j .L104
.L106:
.L107:
addi x5, x5, 1
j .L101
.L108:
j .L107
.L109:
li x10, 1
li x11, 4
call __join_threads
j .L75
.L110:
mv x5, zero
.L111:
bge x5, x19, .L119
slli x12, x5, 12
add ra, x24, x12
lw x11, 0(ra)
beq x11, zero, .L117
mv ra, zero
.L114:
bge ra, x22, .L117
slli x7, ra, 2
add x6, x20, x7
add x10, x6, x12
add x6, x21, x7
add x6, x6, x23
addi ra, ra, 1
lw x6, 0(x6)
mul x7, x11, x6
lw x6, 0(x10)
add x6, x6, x7
sw x6, 0(x10)
j .L114
.L116:
.L117:
addi x5, x5, 1
j .L111
.L118:
j .L117
.L119:
mv x10, zero
li x11, 4
call __join_threads
j .L75
.L120:
li ra, 100
blt ra, x22, .L227
mv x8, zero
.L122:
bge x8, x22, .L148
slli x23, x8, 12
li ra, 1000
blt ra, x22, .L128
mv ra, zero
.L125:
bge ra, x22, .L127
slli x5, ra, 2
add x5, x21, x5
add x5, x5, x23
addi ra, ra, 1
sw zero, 0(x5)
j .L125
.L127:
addi x8, x8, 1
j .L122
.L128:
li x10, 4
call __create_threads
beq x10, zero, .L144
li ra, 1
beq x10, ra, .L140
li ra, 2
beq x10, ra, .L136
lw ra, 8(sp)
.L132:
bge ra, x22, .L134
slli x5, ra, 2
add x5, x21, x5
add x5, x5, x23
addi ra, ra, 1
sw zero, 0(x5)
j .L132
.L134:
li x10, 3
li x11, 4
call __join_threads
.L135:
j .L127
.L136:
lw ra, 4(sp)
.L137:
bge ra, x9, .L139
slli x5, ra, 2
add x5, x21, x5
add x5, x5, x23
addi ra, ra, 1
sw zero, 0(x5)
j .L137
.L139:
li x10, 2
li x11, 4
call __join_threads
j .L127
.L140:
mv ra, x19
.L141:
bge ra, x18, .L143
slli x5, ra, 2
add x5, x21, x5
add x5, x5, x23
addi ra, ra, 1
sw zero, 0(x5)
j .L141
.L143:
li x10, 1
li x11, 4
call __join_threads
j .L127
.L144:
mv ra, zero
.L145:
bge ra, x19, .L147
slli x5, ra, 2
add x5, x21, x5
add x5, x5, x23
addi ra, ra, 1
sw zero, 0(x5)
j .L145
.L147:
mv x10, zero
li x11, 4
call __join_threads
j .L127
.L148:
mv x8, zero
.L149:
bge x8, x22, .L226
slli x5, x8, 2
la ra, _m6global03a03a1A
add x24, ra, x5
slli x23, x8, 12
li ra, 100
blt ra, x22, .L182
mv x25, zero
.L152:
bge x25, x22, .L181
slli x27, x25, 12
add ra, x24, x27
lw x26, 0(ra)
beq x26, zero, .L159
li ra, 1000
blt ra, x22, .L160
mv ra, zero
.L156:
bge ra, x22, .L159
slli x6, ra, 2
add x5, x21, x6
add x7, x5, x27
add x5, x20, x6
add x5, x5, x23
addi ra, ra, 1
lw x5, 0(x5)
mul x6, x26, x5
lw x5, 0(x7)
add x5, x5, x6
sw x5, 0(x7)
j .L156
.L158:
.L159:
addi x25, x25, 1
j .L152
.L160:
li x10, 4
call __create_threads
beq x10, zero, .L176
li ra, 1
beq x10, ra, .L172
li ra, 2
beq x10, ra, .L168
lw ra, 8(sp)
.L164:
bge ra, x22, .L166
slli x6, ra, 2
add x5, x21, x6
add x7, x5, x27
add x5, x20, x6
add x5, x5, x23
addi ra, ra, 1
lw x5, 0(x5)
mul x6, x26, x5
lw x5, 0(x7)
add x5, x5, x6
sw x5, 0(x7)
j .L164
.L166:
li x10, 3
li x11, 4
call __join_threads
.L167:
j .L159
.L168:
lw ra, 4(sp)
.L169:
bge ra, x9, .L171
slli x6, ra, 2
add x5, x21, x6
add x7, x5, x27
add x5, x20, x6
add x5, x5, x23
addi ra, ra, 1
lw x5, 0(x5)
mul x6, x26, x5
lw x5, 0(x7)
add x5, x5, x6
sw x5, 0(x7)
j .L169
.L171:
li x10, 2
li x11, 4
call __join_threads
j .L159
.L172:
mv ra, x19
.L173:
bge ra, x18, .L175
slli x6, ra, 2
add x5, x21, x6
add x7, x5, x27
add x5, x20, x6
add x5, x5, x23
addi ra, ra, 1
lw x5, 0(x5)
mul x6, x26, x5
lw x5, 0(x7)
add x5, x5, x6
sw x5, 0(x7)
j .L173
.L175:
li x10, 1
li x11, 4
call __join_threads
j .L159
.L176:
mv ra, zero
.L177:
bge ra, x19, .L179
slli x6, ra, 2
add x5, x21, x6
add x7, x5, x27
add x5, x20, x6
add x5, x5, x23
addi ra, ra, 1
lw x5, 0(x5)
mul x6, x26, x5
lw x5, 0(x7)
add x5, x5, x6
sw x5, 0(x7)
j .L177
.L179:
mv x10, zero
li x11, 4
call __join_threads
j .L159
.L180:
j .L159
.L181:
addi x8, x8, 1
j .L149
.L182:
li x10, 4
call __create_threads
beq x10, zero, .L216
li ra, 1
beq x10, ra, .L206
li ra, 2
beq x10, ra, .L196
lw x5, 8(sp)
.L186:
bge x5, x22, .L194
slli x12, x5, 12
add ra, x24, x12
lw x11, 0(ra)
beq x11, zero, .L192
mv ra, zero
.L189:
bge ra, x22, .L192
slli x7, ra, 2
add x6, x21, x7
add x10, x6, x12
add x6, x20, x7
add x6, x6, x23
addi ra, ra, 1
lw x6, 0(x6)
mul x7, x11, x6
lw x6, 0(x10)
add x6, x6, x7
sw x6, 0(x10)
j .L189
.L191:
.L192:
addi x5, x5, 1
j .L186
.L193:
j .L192
.L194:
li x10, 3
li x11, 4
call __join_threads
.L195:
j .L181
.L196:
lw x5, 4(sp)
.L197:
bge x5, x9, .L205
slli x12, x5, 12
add ra, x24, x12
lw x11, 0(ra)
beq x11, zero, .L203
mv ra, zero
.L200:
bge ra, x22, .L203
slli x7, ra, 2
add x6, x21, x7
add x10, x6, x12
add x6, x20, x7
add x6, x6, x23
addi ra, ra, 1
lw x6, 0(x6)
mul x7, x11, x6
lw x6, 0(x10)
add x6, x6, x7
sw x6, 0(x10)
j .L200
.L202:
.L203:
addi x5, x5, 1
j .L197
.L204:
j .L203
.L205:
li x10, 2
li x11, 4
call __join_threads
j .L181
.L206:
mv x5, x19
.L207:
bge x5, x18, .L215
slli x12, x5, 12
add ra, x24, x12
lw x11, 0(ra)
beq x11, zero, .L213
mv ra, zero
.L210:
bge ra, x22, .L213
slli x7, ra, 2
add x6, x21, x7
add x10, x6, x12
add x6, x20, x7
add x6, x6, x23
addi ra, ra, 1
lw x6, 0(x6)
mul x7, x11, x6
lw x6, 0(x10)
add x6, x6, x7
sw x6, 0(x10)
j .L210
.L212:
.L213:
addi x5, x5, 1
j .L207
.L214:
j .L213
.L215:
li x10, 1
li x11, 4
call __join_threads
j .L181
.L216:
mv x5, zero
.L217:
bge x5, x19, .L225
slli x12, x5, 12
add ra, x24, x12
lw x11, 0(ra)
beq x11, zero, .L223
mv ra, zero
.L220:
bge ra, x22, .L223
slli x7, ra, 2
add x6, x21, x7
add x10, x6, x12
add x6, x20, x7
add x6, x6, x23
addi ra, ra, 1
lw x6, 0(x6)
mul x7, x11, x6
lw x6, 0(x10)
add x6, x6, x7
sw x6, 0(x10)
j .L220
.L222:
.L223:
addi x5, x5, 1
j .L217
.L224:
j .L223
.L225:
mv x10, zero
li x11, 4
call __join_threads
j .L181
.L226:
lw ra, 0(sp)
addi ra, ra, 1
sw ra, 0(sp)
j .L13
.L227:
li x10, 4
call __create_threads
beq x10, zero, .L252
li ra, 1
beq x10, ra, .L245
li ra, 2
beq x10, ra, .L238
lw x5, 8(sp)
.L231:
bge x5, x22, .L236
slli x7, x5, 12
mv ra, zero
.L233:
bge ra, x22, .L235
slli x6, ra, 2
add x6, x21, x6
add x6, x6, x7
addi ra, ra, 1
sw zero, 0(x6)
j .L233
.L235:
addi x5, x5, 1
j .L231
.L236:
li x10, 3
li x11, 4
call __join_threads
.L237:
j .L148
.L238:
lw x5, 4(sp)
.L239:
bge x5, x9, .L244
slli x7, x5, 12
mv ra, zero
.L241:
bge ra, x22, .L243
slli x6, ra, 2
add x6, x21, x6
add x6, x6, x7
addi ra, ra, 1
sw zero, 0(x6)
j .L241
.L243:
addi x5, x5, 1
j .L239
.L244:
li x10, 2
li x11, 4
call __join_threads
j .L148
.L245:
mv x5, x19
.L246:
bge x5, x18, .L251
slli x7, x5, 12
mv ra, zero
.L248:
bge ra, x22, .L250
slli x6, ra, 2
add x6, x21, x6
add x6, x6, x7
addi ra, ra, 1
sw zero, 0(x6)
j .L248
.L250:
addi x5, x5, 1
j .L246
.L251:
li x10, 1
li x11, 4
call __join_threads
j .L148
.L252:
mv x5, zero
.L253:
bge x5, x19, .L258
slli x7, x5, 12
mv ra, zero
.L255:
bge ra, x22, .L257
slli x6, ra, 2
add x6, x21, x6
add x6, x6, x7
addi ra, ra, 1
sw zero, 0(x6)
j .L255
.L257:
addi x5, x5, 1
j .L253
.L258:
mv x10, zero
li x11, 4
call __join_threads
j .L148
.L259:
li x10, 4
call __create_threads
beq x10, zero, .L284
li ra, 1
beq x10, ra, .L277
li ra, 2
beq x10, ra, .L270
lw x5, 8(sp)
.L263:
bge x5, x22, .L268
slli x7, x5, 12
mv ra, zero
.L265:
bge ra, x22, .L267
slli x6, ra, 2
add x6, x20, x6
add x6, x6, x7
addi ra, ra, 1
sw zero, 0(x6)
j .L265
.L267:
addi x5, x5, 1
j .L263
.L268:
li x10, 3
li x11, 4
call __join_threads
.L269:
j .L42
.L270:
lw x5, 4(sp)
.L271:
bge x5, x9, .L276
slli x7, x5, 12
mv ra, zero
.L273:
bge ra, x22, .L275
slli x6, ra, 2
add x6, x20, x6
add x6, x6, x7
addi ra, ra, 1
sw zero, 0(x6)
j .L273
.L275:
addi x5, x5, 1
j .L271
.L276:
li x10, 2
li x11, 4
call __join_threads
j .L42
.L277:
mv x5, x19
.L278:
bge x5, x18, .L283
slli x7, x5, 12
mv ra, zero
.L280:
bge ra, x22, .L282
slli x6, ra, 2
add x6, x20, x6
add x6, x6, x7
addi ra, ra, 1
sw zero, 0(x6)
j .L280
.L282:
addi x5, x5, 1
j .L278
.L283:
li x10, 1
li x11, 4
call __join_threads
j .L42
.L284:
mv x5, zero
.L285:
bge x5, x19, .L290
slli x7, x5, 12
mv ra, zero
.L287:
bge ra, x22, .L289
slli x6, ra, 2
add x6, x20, x6
add x6, x6, x7
addi ra, ra, 1
sw zero, 0(x6)
j .L287
.L289:
addi x5, x5, 1
j .L285
.L290:
mv x10, zero
li x11, 4
call __join_threads
j .L42
.L291:
la x23, _m6global03a03a4main03a03a12ans_4_reduce
mv x20, zero
mv x8, zero
.L292:
bge x20, x22, .L318
slli x24, x20, 12
li ra, 1000
blt ra, x22, .L298
mv x5, zero
.L295:
mv ra, x8
bge x5, x22, .L297
slli ra, x5, 2
add ra, x21, ra
add ra, ra, x24
addi x5, x5, 1
lw ra, 0(ra)
add x8, x8, ra
j .L295
.L297:
addi x20, x20, 1
mv x8, ra
j .L292
.L298:
li x10, 4
call __create_threads
beq x10, zero, .L314
li ra, 1
beq x10, ra, .L310
li ra, 2
beq x10, ra, .L306
lw x5, 8(sp)
mv ra, zero
.L302:
bge x5, x22, .L304
slli x6, x5, 2
add x6, x21, x6
add x6, x6, x24
addi x5, x5, 1
lw x6, 0(x6)
add ra, ra, x6
j .L302
.L304:
sw ra, 12(x23)
li x10, 3
li x11, 4
call __join_threads
.L305:
lw ra, 0(x23)
add x5, x8, ra
lw ra, 4(x23)
add x5, x5, ra
lw ra, 8(x23)
add x5, x5, ra
lw ra, 12(x23)
add ra, x5, ra
j .L297
.L306:
lw x5, 4(sp)
mv ra, zero
.L307:
bge x5, x9, .L309
slli x6, x5, 2
add x6, x21, x6
add x6, x6, x24
addi x5, x5, 1
lw x6, 0(x6)
add ra, ra, x6
j .L307
.L309:
sw ra, 8(x23)
li x10, 2
li x11, 4
call __join_threads
j .L305
.L310:
mv x5, x19
mv ra, zero
.L311:
bge x5, x18, .L313
slli x6, x5, 2
add x6, x21, x6
add x6, x6, x24
addi x5, x5, 1
lw x6, 0(x6)
add ra, ra, x6
j .L311
.L313:
sw ra, 4(x23)
li x10, 1
li x11, 4
call __join_threads
j .L305
.L314:
mv x5, zero
mv ra, zero
.L315:
bge x5, x19, .L317
slli x6, x5, 2
add x6, x21, x6
add x6, x6, x24
addi x5, x5, 1
lw x6, 0(x6)
add ra, ra, x6
j .L315
.L317:
sw ra, 0(x23)
mv x10, zero
li x11, 4
call __join_threads
j .L305
.L318:
li x10, 84
call _sysy_stoptime
mv x10, x8
call putint
li x10, 10
call putch
mv x10, zero
lw ra, 12(sp)
lw x8, 16(sp)
lw x9, 20(sp)
lw x18, 24(sp)
lw x19, 28(sp)
lw x20, 32(sp)
lw x21, 36(sp)
lw x22, 40(sp)
lw x23, 44(sp)
lw x24, 48(sp)
lw x25, 52(sp)
lw x26, 56(sp)
lw x27, 60(sp)
addi sp, sp, 64
ret


.text
.global __create_threads
.global __join_threads
/*
For system call ABI, see https://man7.org/linux/man-pages/man2/syscall.2.html
*/

/*
Raw system call interface varies on different architectures for clone,
but the manual page (https://man7.org/linux/man-pages/man2/clone.2.html) didn't
mention risc-v. By looking into the kernel source, I figure out that it is
long clone(unsigned long flags, void *stack,
                     int *parent_tid, unsigned long tls,
                     int *child_tid);

int __create_threads(int n) {
    --n;
    if (n <= 0) {
        return 0;
    }
    for (int i = 0; i < n; ++i) {
        int pid = clone(CLONE_VM | SIGCHLD, sp, 0, 0, 0);
        if (pid != 0) {
            return i;
        }
    }
    return n;
}
*/
SYS_clone = 220
CLONE_VM = 256
SIGCHLD = 17
__create_threads:
    addi a0, a0, -1
    ble a0, zero, .ret_0
    mv a6, a0
    li a5, 0
    mv a1, sp
    li a2, 0
    li a3, 0
    li a4, 0
.L0q:
    li a0, (CLONE_VM | SIGCHLD)
    li a7, SYS_clone
    ecall
    bne a0, zero, .ret_i
    addi a5, a5, 1
    blt a5, a6, .L0q
.ret_n:
    mv a0, a6
    j .L1q
.ret_0:
    mv a0, zero
    j .L1q
.ret_i:
    mv a0, a5
.L1q:
    jr ra

/*
Note that it depends on an inconsistent feature between linux and POSIX,
see section BUGS at https://man7.org/linux/man-pages/man2/wait.2.html
But since it already depends on so many features of linux, like the raw
syscall number, so never mind.
void __join_threads(int i, int n) {
    --n;
    if (i != n) {
        waitid(P_ALL, 0, NULL, WEXITED);
    }
    if (i != 0) {
        _exit(0);
    }
}
*/
SYS_waitid = 95
SYS_exit = 93
P_ALL = 0
WEXITED = 4
__join_threads:
    mv a4, a0
    addi a5, a1, -1
    beq a4, a5, .L2q
    li a0, P_ALL
    li a1, 0
    li a2, 0
    li a3, WEXITED
    li a7, SYS_waitid
    ecall
.L2q:
    beq a4, zero, .L3q
    li a0, 0
    li a7, SYS_exit
    ecall
.L3q:
    jr ra
