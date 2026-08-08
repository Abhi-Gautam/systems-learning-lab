	.build_version macos, 26, 0	sdk_version 26, 5
	.section	__TEXT,__text,regular,pure_instructions
	.globl	_add                            ; -- Begin function add
	.p2align	2
_add:                                   ; @add
	.cfi_startproc
; %bb.0:
	add	w0, w1, w0
	ret
	.cfi_endproc
                                        ; -- End function
	.globl	_twice_add                      ; -- Begin function twice_add
	.p2align	2
_twice_add:                             ; @twice_add
	.cfi_startproc
; %bb.0:
	mov	x1, x0
	b	_add
	.cfi_endproc
                                        ; -- End function
	.globl	_main                           ; -- Begin function main
	.p2align	2
_main:                                  ; @main
	.cfi_startproc
; %bb.0:
	mov	w0, #21                         ; =0x15
	b	_twice_add
	.cfi_endproc
                                        ; -- End function
.subsections_via_symbols
