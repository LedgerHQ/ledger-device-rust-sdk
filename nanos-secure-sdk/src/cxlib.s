
@ MACHINE GENERATED -- DO NOT EDIT

.thumb_func

@ --- _cx_trampoline ---
.section .text._cx_trampoline
.func _cx_trampoline
.weak _cx_trampoline
_cx_trampoline:
    MOV R12, R0
    POP {R0}
    BX R12
.endfunc


@ --- cx_hash_get_size ---
.section .text.cx_hash_get_size
.thumb
.thumb_func
.func cx_hash_get_size
.weak cx_hash_get_size
cx_hash_get_size:
  PUSH {R0}
  LDR R0, =#0x122ef5
  B _cx_trampoline
.endfunc
          
@ --- cx_hash_no_throw ---
.section .text.cx_hash_no_throw
.thumb
.thumb_func
.func cx_hash_no_throw
.weak cx_hash_no_throw
cx_hash_no_throw:
  PUSH {R0}
  LDR R0, =#0x122fd7
  B _cx_trampoline
.endfunc
          
@ --- cx_hash_init ---
.section .text.cx_hash_init
.thumb
.thumb_func
.func cx_hash_init
.weak cx_hash_init
cx_hash_init:
  PUSH {R0}
  LDR R0, =#0x122f11
  B _cx_trampoline
.endfunc
          
@ --- cx_hash_init_ex ---
.section .text.cx_hash_init_ex
.thumb
.thumb_func
.func cx_hash_init_ex
.weak cx_hash_init_ex
cx_hash_init_ex:
  PUSH {R0}
  LDR R0, =#0x122f39
  B _cx_trampoline
.endfunc
          
@ --- cx_hash_update ---
.section .text.cx_hash_update
.thumb
.thumb_func
.func cx_hash_update
.weak cx_hash_update
cx_hash_update:
  PUSH {R0}
  LDR R0, =#0x122f71
  B _cx_trampoline
.endfunc
          
@ --- cx_hash_final ---
.section .text.cx_hash_final
.thumb
.thumb_func
.func cx_hash_final
.weak cx_hash_final
cx_hash_final:
  PUSH {R0}
  LDR R0, =#0x122fa3
  B _cx_trampoline
.endfunc
          
@ --- cx_ripemd160_init_no_throw ---
.section .text.cx_ripemd160_init_no_throw
.thumb
.thumb_func
.func cx_ripemd160_init_no_throw
.weak cx_ripemd160_init_no_throw
cx_ripemd160_init_no_throw:
  PUSH {R0}
  LDR R0, =#0x123c71
  B _cx_trampoline
.endfunc
          
@ --- cx_sha224_init_no_throw ---
.section .text.cx_sha224_init_no_throw
.thumb
.thumb_func
.func cx_sha224_init_no_throw
.weak cx_sha224_init_no_throw
cx_sha224_init_no_throw:
  PUSH {R0}
  LDR R0, =#0x1242a1
  B _cx_trampoline
.endfunc
          
@ --- cx_sha256_init_no_throw ---
.section .text.cx_sha256_init_no_throw
.thumb
.thumb_func
.func cx_sha256_init_no_throw
.weak cx_sha256_init_no_throw
cx_sha256_init_no_throw:
  PUSH {R0}
  LDR R0, =#0x1242c5
  B _cx_trampoline
.endfunc
          
@ --- cx_hash_sha256 ---
.section .text.cx_hash_sha256
.thumb
.thumb_func
.func cx_hash_sha256
.weak cx_hash_sha256
cx_hash_sha256:
  PUSH {R0}
  LDR R0, =#0x124409
  B _cx_trampoline
.endfunc
          
@ --- cx_sha384_init_no_throw ---
.section .text.cx_sha384_init_no_throw
.thumb
.thumb_func
.func cx_sha384_init_no_throw
.weak cx_sha384_init_no_throw
cx_sha384_init_no_throw:
  PUSH {R0}
  LDR R0, =#0x1248ad
  B _cx_trampoline
.endfunc
          
@ --- cx_sha512_init_no_throw ---
.section .text.cx_sha512_init_no_throw
.thumb
.thumb_func
.func cx_sha512_init_no_throw
.weak cx_sha512_init_no_throw
cx_sha512_init_no_throw:
  PUSH {R0}
  LDR R0, =#0x1248d1
  B _cx_trampoline
.endfunc
          
@ --- cx_hash_sha512 ---
.section .text.cx_hash_sha512
.thumb
.thumb_func
.func cx_hash_sha512
.weak cx_hash_sha512
cx_hash_sha512:
  PUSH {R0}
  LDR R0, =#0x124a15
  B _cx_trampoline
.endfunc
          
@ --- cx_sha3_init_no_throw ---
.section .text.cx_sha3_init_no_throw
.thumb
.thumb_func
.func cx_sha3_init_no_throw
.weak cx_sha3_init_no_throw
cx_sha3_init_no_throw:
  PUSH {R0}
  LDR R0, =#0x124439
  B _cx_trampoline
.endfunc
          
@ --- cx_keccak_init_no_throw ---
.section .text.cx_keccak_init_no_throw
.thumb
.thumb_func
.func cx_keccak_init_no_throw
.weak cx_keccak_init_no_throw
cx_keccak_init_no_throw:
  PUSH {R0}
  LDR R0, =#0x124475
  B _cx_trampoline
.endfunc
          
@ --- cx_shake128_init_no_throw ---
.section .text.cx_shake128_init_no_throw
.thumb
.thumb_func
.func cx_shake128_init_no_throw
.weak cx_shake128_init_no_throw
cx_shake128_init_no_throw:
  PUSH {R0}
  LDR R0, =#0x12449b
  B _cx_trampoline
.endfunc
          
@ --- cx_shake256_init_no_throw ---
.section .text.cx_shake256_init_no_throw
.thumb
.thumb_func
.func cx_shake256_init_no_throw
.weak cx_shake256_init_no_throw
cx_shake256_init_no_throw:
  PUSH {R0}
  LDR R0, =#0x1244c5
  B _cx_trampoline
.endfunc
          
@ --- cx_sha3_xof_init_no_throw ---
.section .text.cx_sha3_xof_init_no_throw
.thumb
.thumb_func
.func cx_sha3_xof_init_no_throw
.weak cx_sha3_xof_init_no_throw
cx_sha3_xof_init_no_throw:
  PUSH {R0}
  LDR R0, =#0x1244ef
  B _cx_trampoline
.endfunc
          
@ --- cx_blake2b_init_no_throw ---
.section .text.cx_blake2b_init_no_throw
.thumb
.thumb_func
.func cx_blake2b_init_no_throw
.weak cx_blake2b_init_no_throw
cx_blake2b_init_no_throw:
  PUSH {R0}
  LDR R0, =#0x1208cb
  B _cx_trampoline
.endfunc
          
@ --- cx_blake2b_init2_no_throw ---
.section .text.cx_blake2b_init2_no_throw
.thumb
.thumb_func
.func cx_blake2b_init2_no_throw
.weak cx_blake2b_init2_no_throw
cx_blake2b_init2_no_throw:
  PUSH {R0}
  LDR R0, =#0x120841
  B _cx_trampoline
.endfunc
          
@ --- cx_groestl_init_no_throw ---
.section .text.cx_groestl_init_no_throw
.thumb
.thumb_func
.func cx_groestl_init_no_throw
.weak cx_groestl_init_no_throw
cx_groestl_init_no_throw:
  PUSH {R0}
  LDR R0, =#0x120001
  B _cx_trampoline
.endfunc
          
@ --- cx_hmac_ripemd160_init_no_throw ---
.section .text.cx_hmac_ripemd160_init_no_throw
.thumb
.thumb_func
.func cx_hmac_ripemd160_init_no_throw
.weak cx_hmac_ripemd160_init_no_throw
cx_hmac_ripemd160_init_no_throw:
  PUSH {R0}
  LDR R0, =#0x123301
  B _cx_trampoline
.endfunc
          
@ --- cx_hmac_sha224_init ---
.section .text.cx_hmac_sha224_init
.thumb
.thumb_func
.func cx_hmac_sha224_init
.weak cx_hmac_sha224_init
cx_hmac_sha224_init:
  PUSH {R0}
  LDR R0, =#0x1232c9
  B _cx_trampoline
.endfunc
          
@ --- cx_hmac_sha256_init_no_throw ---
.section .text.cx_hmac_sha256_init_no_throw
.thumb
.thumb_func
.func cx_hmac_sha256_init_no_throw
.weak cx_hmac_sha256_init_no_throw
cx_hmac_sha256_init_no_throw:
  PUSH {R0}
  LDR R0, =#0x1232d7
  B _cx_trampoline
.endfunc
          
@ --- cx_hmac_sha256 ---
.section .text.cx_hmac_sha256
.thumb
.thumb_func
.func cx_hmac_sha256
.weak cx_hmac_sha256
cx_hmac_sha256:
  PUSH {R0}
  LDR R0, =#0x12337d
  B _cx_trampoline
.endfunc
          
@ --- cx_hmac_sha384_init ---
.section .text.cx_hmac_sha384_init
.thumb
.thumb_func
.func cx_hmac_sha384_init
.weak cx_hmac_sha384_init
cx_hmac_sha384_init:
  PUSH {R0}
  LDR R0, =#0x1232e5
  B _cx_trampoline
.endfunc
          
@ --- cx_hmac_sha512_init_no_throw ---
.section .text.cx_hmac_sha512_init_no_throw
.thumb
.thumb_func
.func cx_hmac_sha512_init_no_throw
.weak cx_hmac_sha512_init_no_throw
cx_hmac_sha512_init_no_throw:
  PUSH {R0}
  LDR R0, =#0x1232f3
  B _cx_trampoline
.endfunc
          
@ --- cx_hmac_sha512 ---
.section .text.cx_hmac_sha512
.thumb
.thumb_func
.func cx_hmac_sha512
.weak cx_hmac_sha512
cx_hmac_sha512:
  PUSH {R0}
  LDR R0, =#0x1233b1
  B _cx_trampoline
.endfunc
          
@ --- cx_hmac_no_throw ---
.section .text.cx_hmac_no_throw
.thumb
.thumb_func
.func cx_hmac_no_throw
.weak cx_hmac_no_throw
cx_hmac_no_throw:
  PUSH {R0}
  LDR R0, =#0x12330f
  B _cx_trampoline
.endfunc
          
@ --- cx_hmac_init ---
.section .text.cx_hmac_init
.thumb
.thumb_func
.func cx_hmac_init
.weak cx_hmac_init
cx_hmac_init:
  PUSH {R0}
  LDR R0, =#0x123147
  B _cx_trampoline
.endfunc
          
@ --- cx_hmac_update ---
.section .text.cx_hmac_update
.thumb
.thumb_func
.func cx_hmac_update
.weak cx_hmac_update
cx_hmac_update:
  PUSH {R0}
  LDR R0, =#0x1231e7
  B _cx_trampoline
.endfunc
          
@ --- cx_hmac_final ---
.section .text.cx_hmac_final
.thumb
.thumb_func
.func cx_hmac_final
.weak cx_hmac_final
cx_hmac_final:
  PUSH {R0}
  LDR R0, =#0x12321b
  B _cx_trampoline
.endfunc
          
@ --- cx_pbkdf2_no_throw ---
.section .text.cx_pbkdf2_no_throw
.thumb
.thumb_func
.func cx_pbkdf2_no_throw
.weak cx_pbkdf2_no_throw
cx_pbkdf2_no_throw:
  PUSH {R0}
  LDR R0, =#0x123a95
  B _cx_trampoline
.endfunc
          
@ --- cx_rng_u32_range_func ---
.section .text.cx_rng_u32_range_func
.thumb
.thumb_func
.func cx_rng_u32_range_func
.weak cx_rng_u32_range_func
cx_rng_u32_range_func:
  PUSH {R0}
  LDR R0, =#0x123db5
  B _cx_trampoline
.endfunc
          
@ --- cx_rng_u32_range ---
.section .text.cx_rng_u32_range
.thumb
.thumb_func
.func cx_rng_u32_range
.weak cx_rng_u32_range
cx_rng_u32_range:
  PUSH {R0}
  LDR R0, =#0x123ded
  B _cx_trampoline
.endfunc
          
@ --- cx_rng_u32 ---
.section .text.cx_rng_u32
.thumb
.thumb_func
.func cx_rng_u32
.weak cx_rng_u32
cx_rng_u32:
  PUSH {R0}
  LDR R0, =#0x123d95
  B _cx_trampoline
.endfunc
          
@ --- cx_rng_u8 ---
.section .text.cx_rng_u8
.thumb
.thumb_func
.func cx_rng_u8
.weak cx_rng_u8
cx_rng_u8:
  PUSH {R0}
  LDR R0, =#0x123da3
  B _cx_trampoline
.endfunc
          
@ --- cx_rng_no_throw ---
.section .text.cx_rng_no_throw
.thumb
.thumb_func
.func cx_rng_no_throw
.weak cx_rng_no_throw
cx_rng_no_throw:
  PUSH {R0}
  LDR R0, =#0x123d87
  B _cx_trampoline
.endfunc
          
@ --- cx_rng_rfc6979 ---
.section .text.cx_rng_rfc6979
.thumb
.thumb_func
.func cx_rng_rfc6979
.weak cx_rng_rfc6979
cx_rng_rfc6979:
  PUSH {R0}
  LDR R0, =#0x12411d
  B _cx_trampoline
.endfunc
          
@ --- cx_math_cmp_no_throw ---
.section .text.cx_math_cmp_no_throw
.thumb
.thumb_func
.func cx_math_cmp_no_throw
.weak cx_math_cmp_no_throw
cx_math_cmp_no_throw:
  PUSH {R0}
  LDR R0, =#0x1233e7
  B _cx_trampoline
.endfunc
          
@ --- cx_math_add_no_throw ---
.section .text.cx_math_add_no_throw
.thumb
.thumb_func
.func cx_math_add_no_throw
.weak cx_math_add_no_throw
cx_math_add_no_throw:
  PUSH {R0}
  LDR R0, =#0x12343b
  B _cx_trampoline
.endfunc
          
@ --- cx_math_sub_no_throw ---
.section .text.cx_math_sub_no_throw
.thumb
.thumb_func
.func cx_math_sub_no_throw
.weak cx_math_sub_no_throw
cx_math_sub_no_throw:
  PUSH {R0}
  LDR R0, =#0x1234b3
  B _cx_trampoline
.endfunc
          
@ --- cx_math_mult_no_throw ---
.section .text.cx_math_mult_no_throw
.thumb
.thumb_func
.func cx_math_mult_no_throw
.weak cx_math_mult_no_throw
cx_math_mult_no_throw:
  PUSH {R0}
  LDR R0, =#0x12352b
  B _cx_trampoline
.endfunc
          
@ --- cx_math_addm_no_throw ---
.section .text.cx_math_addm_no_throw
.thumb
.thumb_func
.func cx_math_addm_no_throw
.weak cx_math_addm_no_throw
cx_math_addm_no_throw:
  PUSH {R0}
  LDR R0, =#0x12359f
  B _cx_trampoline
.endfunc
          
@ --- cx_math_subm_no_throw ---
.section .text.cx_math_subm_no_throw
.thumb
.thumb_func
.func cx_math_subm_no_throw
.weak cx_math_subm_no_throw
cx_math_subm_no_throw:
  PUSH {R0}
  LDR R0, =#0x123625
  B _cx_trampoline
.endfunc
          
@ --- cx_math_multm_no_throw ---
.section .text.cx_math_multm_no_throw
.thumb
.thumb_func
.func cx_math_multm_no_throw
.weak cx_math_multm_no_throw
cx_math_multm_no_throw:
  PUSH {R0}
  LDR R0, =#0x1236ab
  B _cx_trampoline
.endfunc
          
@ --- cx_math_modm_no_throw ---
.section .text.cx_math_modm_no_throw
.thumb
.thumb_func
.func cx_math_modm_no_throw
.weak cx_math_modm_no_throw
cx_math_modm_no_throw:
  PUSH {R0}
  LDR R0, =#0x123731
  B _cx_trampoline
.endfunc
          
@ --- cx_math_powm_no_throw ---
.section .text.cx_math_powm_no_throw
.thumb
.thumb_func
.func cx_math_powm_no_throw
.weak cx_math_powm_no_throw
cx_math_powm_no_throw:
  PUSH {R0}
  LDR R0, =#0x1237a3
  B _cx_trampoline
.endfunc
          
@ --- cx_math_invprimem_no_throw ---
.section .text.cx_math_invprimem_no_throw
.thumb
.thumb_func
.func cx_math_invprimem_no_throw
.weak cx_math_invprimem_no_throw
cx_math_invprimem_no_throw:
  PUSH {R0}
  LDR R0, =#0x12382b
  B _cx_trampoline
.endfunc
          
@ --- cx_math_invintm_no_throw ---
.section .text.cx_math_invintm_no_throw
.thumb
.thumb_func
.func cx_math_invintm_no_throw
.weak cx_math_invintm_no_throw
cx_math_invintm_no_throw:
  PUSH {R0}
  LDR R0, =#0x12389b
  B _cx_trampoline
.endfunc
          
@ --- cx_math_is_prime_no_throw ---
.section .text.cx_math_is_prime_no_throw
.thumb
.thumb_func
.func cx_math_is_prime_no_throw
.weak cx_math_is_prime_no_throw
cx_math_is_prime_no_throw:
  PUSH {R0}
  LDR R0, =#0x1238f9
  B _cx_trampoline
.endfunc
          
@ --- cx_math_next_prime_no_throw ---
.section .text.cx_math_next_prime_no_throw
.thumb
.thumb_func
.func cx_math_next_prime_no_throw
.weak cx_math_next_prime_no_throw
cx_math_next_prime_no_throw:
  PUSH {R0}
  LDR R0, =#0x123935
  B _cx_trampoline
.endfunc
          
@ --- cx_des_init_key_no_throw ---
.section .text.cx_des_init_key_no_throw
.thumb
.thumb_func
.func cx_des_init_key_no_throw
.weak cx_des_init_key_no_throw
cx_des_init_key_no_throw:
  PUSH {R0}
  LDR R0, =#0x12060d
  B _cx_trampoline
.endfunc
          
@ --- cx_des_iv_no_throw ---
.section .text.cx_des_iv_no_throw
.thumb
.thumb_func
.func cx_des_iv_no_throw
.weak cx_des_iv_no_throw
cx_des_iv_no_throw:
  PUSH {R0}
  LDR R0, =#0x120689
  B _cx_trampoline
.endfunc
          
@ --- cx_des_no_throw ---
.section .text.cx_des_no_throw
.thumb
.thumb_func
.func cx_des_no_throw
.weak cx_des_no_throw
cx_des_no_throw:
  PUSH {R0}
  LDR R0, =#0x1206f9
  B _cx_trampoline
.endfunc
          
@ --- cx_des_enc_block ---
.section .text.cx_des_enc_block
.thumb
.thumb_func
.func cx_des_enc_block
.weak cx_des_enc_block
cx_des_enc_block:
  PUSH {R0}
  LDR R0, =#0x1207e5
  B _cx_trampoline
.endfunc
          
@ --- cx_des_dec_block ---
.section .text.cx_des_dec_block
.thumb
.thumb_func
.func cx_des_dec_block
.weak cx_des_dec_block
cx_des_dec_block:
  PUSH {R0}
  LDR R0, =#0x1207ff
  B _cx_trampoline
.endfunc
          
@ --- cx_aes_init_key_no_throw ---
.section .text.cx_aes_init_key_no_throw
.thumb
.thumb_func
.func cx_aes_init_key_no_throw
.weak cx_aes_init_key_no_throw
cx_aes_init_key_no_throw:
  PUSH {R0}
  LDR R0, =#0x120649
  B _cx_trampoline
.endfunc
          
@ --- cx_aes_iv_no_throw ---
.section .text.cx_aes_iv_no_throw
.thumb
.thumb_func
.func cx_aes_iv_no_throw
.weak cx_aes_iv_no_throw
cx_aes_iv_no_throw:
  PUSH {R0}
  LDR R0, =#0x120713
  B _cx_trampoline
.endfunc
          
@ --- cx_aes_no_throw ---
.section .text.cx_aes_no_throw
.thumb
.thumb_func
.func cx_aes_no_throw
.weak cx_aes_no_throw
cx_aes_no_throw:
  PUSH {R0}
  LDR R0, =#0x120783
  B _cx_trampoline
.endfunc
          
@ --- cx_aes_enc_block ---
.section .text.cx_aes_enc_block
.thumb
.thumb_func
.func cx_aes_enc_block
.weak cx_aes_enc_block
cx_aes_enc_block:
  PUSH {R0}
  LDR R0, =#0x12079d
  B _cx_trampoline
.endfunc
          
@ --- cx_aes_dec_block ---
.section .text.cx_aes_dec_block
.thumb
.thumb_func
.func cx_aes_dec_block
.weak cx_aes_dec_block
cx_aes_dec_block:
  PUSH {R0}
  LDR R0, =#0x1207c1
  B _cx_trampoline
.endfunc
          
@ --- cx_ecfp_add_point_no_throw ---
.section .text.cx_ecfp_add_point_no_throw
.thumb
.thumb_func
.func cx_ecfp_add_point_no_throw
.weak cx_ecfp_add_point_no_throw
cx_ecfp_add_point_no_throw:
  PUSH {R0}
  LDR R0, =#0x1214af
  B _cx_trampoline
.endfunc
          
@ --- cx_ecfp_scalar_mult_no_throw ---
.section .text.cx_ecfp_scalar_mult_no_throw
.thumb
.thumb_func
.func cx_ecfp_scalar_mult_no_throw
.weak cx_ecfp_scalar_mult_no_throw
cx_ecfp_scalar_mult_no_throw:
  PUSH {R0}
  LDR R0, =#0x12154d
  B _cx_trampoline
.endfunc
          
@ --- cx_ecfp_init_public_key_no_throw ---
.section .text.cx_ecfp_init_public_key_no_throw
.thumb
.thumb_func
.func cx_ecfp_init_public_key_no_throw
.weak cx_ecfp_init_public_key_no_throw
cx_ecfp_init_public_key_no_throw:
  PUSH {R0}
  LDR R0, =#0x121251
  B _cx_trampoline
.endfunc
          
@ --- cx_ecfp_init_private_key_no_throw ---
.section .text.cx_ecfp_init_private_key_no_throw
.thumb
.thumb_func
.func cx_ecfp_init_private_key_no_throw
.weak cx_ecfp_init_private_key_no_throw
cx_ecfp_init_private_key_no_throw:
  PUSH {R0}
  LDR R0, =#0x12120d
  B _cx_trampoline
.endfunc
          
@ --- cx_ecfp_generate_pair_no_throw ---
.section .text.cx_ecfp_generate_pair_no_throw
.thumb
.thumb_func
.func cx_ecfp_generate_pair_no_throw
.weak cx_ecfp_generate_pair_no_throw
cx_ecfp_generate_pair_no_throw:
  PUSH {R0}
  LDR R0, =#0x1214a3
  B _cx_trampoline
.endfunc
          
@ --- cx_ecfp_generate_pair2_no_throw ---
.section .text.cx_ecfp_generate_pair2_no_throw
.thumb
.thumb_func
.func cx_ecfp_generate_pair2_no_throw
.weak cx_ecfp_generate_pair2_no_throw
cx_ecfp_generate_pair2_no_throw:
  PUSH {R0}
  LDR R0, =#0x1212dd
  B _cx_trampoline
.endfunc
          
@ --- cx_eddsa_get_public_key_no_throw ---
.section .text.cx_eddsa_get_public_key_no_throw
.thumb
.thumb_func
.func cx_eddsa_get_public_key_no_throw
.weak cx_eddsa_get_public_key_no_throw
cx_eddsa_get_public_key_no_throw:
  PUSH {R0}
  LDR R0, =#0x1226c1
  B _cx_trampoline
.endfunc
          
@ --- cx_edwards_compress_point_no_throw ---
.section .text.cx_edwards_compress_point_no_throw
.thumb
.thumb_func
.func cx_edwards_compress_point_no_throw
.weak cx_edwards_compress_point_no_throw
cx_edwards_compress_point_no_throw:
  PUSH {R0}
  LDR R0, =#0x1215bf
  B _cx_trampoline
.endfunc
          
@ --- cx_edwards_decompress_point_no_throw ---
.section .text.cx_edwards_decompress_point_no_throw
.thumb
.thumb_func
.func cx_edwards_decompress_point_no_throw
.weak cx_edwards_decompress_point_no_throw
cx_edwards_decompress_point_no_throw:
  PUSH {R0}
  LDR R0, =#0x121633
  B _cx_trampoline
.endfunc
          
@ --- cx_ecdsa_sign_no_throw ---
.section .text.cx_ecdsa_sign_no_throw
.thumb
.thumb_func
.func cx_ecdsa_sign_no_throw
.weak cx_ecdsa_sign_no_throw
cx_ecdsa_sign_no_throw:
  PUSH {R0}
  LDR R0, =#0x120a67
  B _cx_trampoline
.endfunc
          
@ --- cx_ecdsa_verify_no_throw ---
.section .text.cx_ecdsa_verify_no_throw
.thumb
.thumb_func
.func cx_ecdsa_verify_no_throw
.weak cx_ecdsa_verify_no_throw
cx_ecdsa_verify_no_throw:
  PUSH {R0}
  LDR R0, =#0x120eeb
  B _cx_trampoline
.endfunc
          
@ --- cx_eddsa_sign_no_throw ---
.section .text.cx_eddsa_sign_no_throw
.thumb
.thumb_func
.func cx_eddsa_sign_no_throw
.weak cx_eddsa_sign_no_throw
cx_eddsa_sign_no_throw:
  PUSH {R0}
  LDR R0, =#0x122711
  B _cx_trampoline
.endfunc
          
@ --- cx_eddsa_verify_no_throw ---
.section .text.cx_eddsa_verify_no_throw
.thumb
.thumb_func
.func cx_eddsa_verify_no_throw
.weak cx_eddsa_verify_no_throw
cx_eddsa_verify_no_throw:
  PUSH {R0}
  LDR R0, =#0x122a79
  B _cx_trampoline
.endfunc
          
@ --- cx_encode_coord ---
.section .text.cx_encode_coord
.thumb
.thumb_func
.func cx_encode_coord
.weak cx_encode_coord
cx_encode_coord:
  PUSH {R0}
  LDR R0, =#0x12252d
  B _cx_trampoline
.endfunc
          
@ --- cx_decode_coord ---
.section .text.cx_decode_coord
.thumb
.thumb_func
.func cx_decode_coord
.weak cx_decode_coord
cx_decode_coord:
  PUSH {R0}
  LDR R0, =#0x122517
  B _cx_trampoline
.endfunc
          
@ --- cx_ecschnorr_sign_no_throw ---
.section .text.cx_ecschnorr_sign_no_throw
.thumb
.thumb_func
.func cx_ecschnorr_sign_no_throw
.weak cx_ecschnorr_sign_no_throw
cx_ecschnorr_sign_no_throw:
  PUSH {R0}
  LDR R0, =#0x121795
  B _cx_trampoline
.endfunc
          
@ --- cx_ecschnorr_verify ---
.section .text.cx_ecschnorr_verify
.thumb
.thumb_func
.func cx_ecschnorr_verify
.weak cx_ecschnorr_verify
cx_ecschnorr_verify:
  PUSH {R0}
  LDR R0, =#0x121e15
  B _cx_trampoline
.endfunc
          
@ --- cx_ecdh_no_throw ---
.section .text.cx_ecdh_no_throw
.thumb
.thumb_func
.func cx_ecdh_no_throw
.weak cx_ecdh_no_throw
cx_ecdh_no_throw:
  PUSH {R0}
  LDR R0, =#0x120985
  B _cx_trampoline
.endfunc
          
@ --- cx_crc16 ---
.section .text.cx_crc16
.thumb
.thumb_func
.func cx_crc16
.weak cx_crc16
cx_crc16:
  PUSH {R0}
  LDR R0, =#0x120969
  B _cx_trampoline
.endfunc
          
@ --- cx_crc16_update ---
.section .text.cx_crc16_update
.thumb
.thumb_func
.func cx_crc16_update
.weak cx_crc16_update
cx_crc16_update:
  PUSH {R0}
  LDR R0, =#0x120945
  B _cx_trampoline
.endfunc
          