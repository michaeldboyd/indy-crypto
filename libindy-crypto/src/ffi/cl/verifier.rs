use cl::verifier::*;
use cl::*;
use errors::ToErrorCode;
use ffi::ErrorCode;
use utils::ctypes::CTypesUtils;

use libc::c_char;

use std::os::raw::c_void;

/// Creates and returns proof verifier.
///
/// Note that proof verifier deallocation must be performed by
/// calling indy_crypto_cl_proof_verifier_finalize.
///
/// # Arguments
/// * `proof_verifier_p` - Reference that will contain proof verifier instance pointer.
#[no_mangle]
pub extern fn indy_crypto_cl_verifier_new_proof_verifier(proof_verifier_p: *mut *const c_void) -> ErrorCode {
    trace!("indy_crypto_cl_verifier_new_proof_verifier: >>> {:?}", proof_verifier_p);

    check_useful_c_ptr!(proof_verifier_p, ErrorCode::CommonInvalidParam1);

    let res = match Verifier::new_proof_verifier() {
        Ok(proof_verifier) => {
            trace!("indy_crypto_cl_verifier_new_proof_verifier: proof_verifier: {:?}", proof_verifier);
            unsafe {
                *proof_verifier_p = Box::into_raw(Box::new(proof_verifier)) as *const c_void;
                trace!("indy_crypto_cl_verifier_new_proof_verifier: *proof_verifier_p: {:?}", *proof_verifier_p);
            }
            ErrorCode::Success
        }
        Err(err) => err.to_error_code()
    };

    trace!("indy_crypto_cl_verifier_new_proof_verifier: <<< res: {:?}", res);
    res
}

#[no_mangle]
pub extern fn indy_crypto_cl_proof_verifier_add_sub_proof_request(proof_verifier: *const c_void,
                                                                  key_id: *const c_char,
                                                                  sub_proof_request: *const c_void,
                                                                  credential_schema: *const c_void,
                                                                  credential_pub_key: *const c_void,
                                                                  rev_key_pub: *const c_void,
                                                                  rev_reg: *const c_void) -> ErrorCode {
    trace!("indy_crypto_cl_proof_verifier_add_sub_proof_request: >>> proof_verifier: {:?}, key_id: {:?}, sub_proof_request: {:?} ,\
                credential_schema: {:?}, credential_pub_key: {:?}, rev_key_pub: {:?}, rev_reg: {:?}",
           proof_verifier, key_id, sub_proof_request, credential_schema, credential_pub_key, rev_key_pub, rev_reg);

    check_useful_mut_c_reference!(proof_verifier, ProofVerifier, ErrorCode::CommonInvalidParam1);
    check_useful_c_str!(key_id, ErrorCode::CommonInvalidParam2);
    check_useful_c_reference!(sub_proof_request, SubProofRequest, ErrorCode::CommonInvalidParam3);
    check_useful_c_reference!(credential_schema, CredentialSchema, ErrorCode::CommonInvalidParam4);
    check_useful_c_reference!(credential_pub_key, CredentialPublicKey, ErrorCode::CommonInvalidParam5);
    check_useful_opt_c_reference!(rev_key_pub, RevocationKeyPublic);
    check_useful_opt_c_reference!(rev_reg, RevocationRegistry);

    trace!("indy_crypto_cl_proof_verifier_add_sub_proof_request: entities: proof_verifier: {:?}, key_id: {:?}, sub_proof_request: {:?},\
                credential_schema: {:?}, credential_pub_key: {:?}, rev_key_pub: {:?}, rev_reg: {:?}",
           proof_verifier, key_id, sub_proof_request, credential_schema, credential_pub_key, rev_key_pub, rev_reg);

    let res = match proof_verifier.add_sub_proof_request(&key_id,
                                                         sub_proof_request,
                                                         credential_schema,
                                                         credential_pub_key,
                                                         rev_key_pub,
                                                         rev_reg) {
        Ok(()) => ErrorCode::Success,
        Err(err) => err.to_error_code()
    };

    trace!("indy_crypto_cl_proof_verifier_add_sub_proof_request: <<< res: {:?}", res);
    ErrorCode::Success
}


/// Verifies proof and deallocates proof verifier.
///
/// # Arguments
/// * `proof_verifier` - Reference that contain proof verifier instance pointer.
/// * `proof` - Reference that contain proof instance pointer.
/// * `nonce` - Reference that contain nonce instance pointer.
/// * `valid_p` - Reference that will be filled with true - if proof valid or false otherwise.
#[no_mangle]
pub extern fn indy_crypto_cl_proof_verifier_verify(proof_verifier: *const c_void,
                                                   proof: *const c_void,
                                                   nonce: *const c_void,
                                                   valid_p: *mut bool) -> ErrorCode {
    trace!("indy_crypto_cl_proof_verifier_verify: >>> proof_verifier: {:?}, proof: {:?}, nonce: {:?}, valid_p: {:?}", proof_verifier, proof, nonce, valid_p);

    check_useful_c_ptr!(proof_verifier, ErrorCode::CommonInvalidParam1);
    check_useful_c_reference!(proof, Proof, ErrorCode::CommonInvalidParam2);
    check_useful_c_reference!(nonce, Nonce, ErrorCode::CommonInvalidParam3);
    check_useful_c_ptr!(valid_p, ErrorCode::CommonInvalidParam4);

    let proof_verifier = unsafe { Box::from_raw(proof_verifier as *mut ProofVerifier) };

    trace!("indy_crypto_cl_proof_verifier_verify: entities: >>> proof_verifier: {:?}, proof: {:?}, nonce: {:?}", proof_verifier, proof, nonce);

    let res = match proof_verifier.verify(proof, nonce) {
        Ok(valid) => {
            trace!("indy_crypto_cl_proof_verifier_verify: valid: {:?}", valid);
            unsafe {
                *valid_p = valid;
                trace!("indy_crypto_cl_proof_verifier_verify: *valid_p: {:?}", *valid_p);
            }
            ErrorCode::Success
        }
        Err(err) => err.to_error_code()
    };

    trace!("indy_crypto_cl_proof_verifier_verify: <<< res: {:?}", res);
    res
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::ffi::CString;
    use std::ptr;
    use ffi::cl::mocks::*;
    use super::mocks::*;
    use super::super::issuer::mocks::*;
    use super::super::prover::mocks::*;

    #[test]
    fn indy_crypto_cl_verifier_new_proof_verifier_works() {
        let key_id = CString::new("key_id").unwrap();
        let (credential_pub_key, credential_priv_key, credential_key_correctness_proof) = _credential_def();
        let master_secret = _master_secret();
        let master_secret_blinding_nonce = _nonce();
        let (blinded_master_secret, master_secret_blinding_data,
            blinded_master_secret_correctness_proof) = _blinded_master_secret(credential_pub_key,
                                                                              credential_key_correctness_proof,
                                                                              master_secret,
                                                                              master_secret_blinding_nonce);
        let credential_issuance_nonce = _nonce();
        let (credential_signature, signature_correctness_proof) = _credential_signature(blinded_master_secret,
                                                                                        blinded_master_secret_correctness_proof,
                                                                                        master_secret_blinding_nonce,
                                                                                        credential_issuance_nonce,
                                                                                        credential_pub_key,
                                                                                        credential_priv_key);
        let credential_schema = _credential_schema();
        let sub_proof_request = _sub_proof_request();
        _process_credential_signature(credential_signature,
                                      signature_correctness_proof,
                                      master_secret_blinding_data,
                                      master_secret,
                                      credential_pub_key,
                                      credential_issuance_nonce,
                                      ptr::null(),
                                      ptr::null(),
                                      ptr::null());

        let proof_building_nonce = _nonce();
        let proof = _proof(credential_pub_key,
                           credential_signature,
                           proof_building_nonce,
                           master_secret,
                           ptr::null(),
                           ptr::null());

        let mut proof_verifier_p: *const c_void = ptr::null();
        let err_code = indy_crypto_cl_verifier_new_proof_verifier(&mut proof_verifier_p);
        assert_eq!(err_code, ErrorCode::Success);
        assert!(!proof_verifier_p.is_null());

        _add_sub_proof_request(proof_verifier_p, key_id, credential_schema, credential_pub_key, sub_proof_request, ptr::null(), ptr::null());
        _free_proof_verifier(proof_verifier_p, proof, proof_building_nonce);
        _free_credential_def(credential_pub_key, credential_priv_key, credential_key_correctness_proof);
        _free_master_secret(master_secret);
        _free_blinded_master_secret(blinded_master_secret, master_secret_blinding_data, blinded_master_secret_correctness_proof);
        _free_nonce(master_secret_blinding_nonce);
        _free_nonce(credential_issuance_nonce);
        _free_nonce(proof_building_nonce);
        _free_credential_schema(credential_schema);
        _free_sub_proof_request(sub_proof_request);
        _free_credential_signature(credential_signature, signature_correctness_proof);
    }

    #[test]
    fn indy_crypto_cl_proof_verifier_add_sub_proof_request_works() {
        let key_id = CString::new("key_id").unwrap();
        let (credential_pub_key, credential_priv_key, credential_key_correctness_proof) = _credential_def();
        let master_secret = _master_secret();
        let master_secret_blinding_nonce = _nonce();
        let (blinded_master_secret, master_secret_blinding_data,
            blinded_master_secret_correctness_proof) = _blinded_master_secret(credential_pub_key,
                                                                              credential_key_correctness_proof,
                                                                              master_secret,
                                                                              master_secret_blinding_nonce);
        let credential_issuance_nonce = _nonce();
        let (credential_signature, signature_correctness_proof) = _credential_signature(blinded_master_secret,
                                                                                        blinded_master_secret_correctness_proof,
                                                                                        master_secret_blinding_nonce,
                                                                                        credential_issuance_nonce,
                                                                                        credential_pub_key,
                                                                                        credential_priv_key);
        let credential_schema = _credential_schema();
        let sub_proof_request = _sub_proof_request();
        _process_credential_signature(credential_signature,
                                      signature_correctness_proof,
                                      master_secret_blinding_data,
                                      master_secret,
                                      credential_pub_key,
                                      credential_issuance_nonce,
                                      ptr::null(),
                                      ptr::null(),
                                      ptr::null());

        let proof_building_nonce = _nonce();
        let proof = _proof(credential_pub_key,
                           credential_signature,
                           proof_building_nonce,
                           master_secret,
                           ptr::null(),
                           ptr::null());

        let proof_verifier = _proof_verifier();

        let err_code = indy_crypto_cl_proof_verifier_add_sub_proof_request(proof_verifier,
                                                                           key_id.as_ptr(),
                                                                           sub_proof_request,
                                                                           credential_schema,
                                                                           credential_pub_key,
                                                                           ptr::null(),
                                                                           ptr::null());
        assert_eq!(err_code, ErrorCode::Success);

        _free_proof_verifier(proof_verifier, proof, proof_building_nonce);
        _free_credential_def(credential_pub_key, credential_priv_key, credential_key_correctness_proof);
        _free_master_secret(master_secret);
        _free_blinded_master_secret(blinded_master_secret, master_secret_blinding_data, blinded_master_secret_correctness_proof);
        _free_nonce(master_secret_blinding_nonce);
        _free_nonce(credential_issuance_nonce);
        _free_nonce(proof_building_nonce);
        _free_credential_schema(credential_schema);
        _free_sub_proof_request(sub_proof_request);
        _free_credential_signature(credential_signature, signature_correctness_proof);
    }

    #[test]
    fn indy_crypto_cl_proof_verifier_verify_works_for_primary_proof() {
        let key_id = CString::new("key_id").unwrap();
        let (credential_pub_key, credential_priv_key, credential_key_correctness_proof) = _credential_def();
        let master_secret = _master_secret();
        let master_secret_blinding_nonce = _nonce();
        let (blinded_master_secret, master_secret_blinding_data,
            blinded_master_secret_correctness_proof) = _blinded_master_secret(credential_pub_key,
                                                                              credential_key_correctness_proof,
                                                                              master_secret,
                                                                              master_secret_blinding_nonce);
        let credential_issuance_nonce = _nonce();
        let (credential_signature, signature_correctness_proof) = _credential_signature(blinded_master_secret,
                                                                                        blinded_master_secret_correctness_proof,
                                                                                        master_secret_blinding_nonce,
                                                                                        credential_issuance_nonce,
                                                                                        credential_pub_key,
                                                                                        credential_priv_key);
        let credential_schema = _credential_schema();
        let sub_proof_request = _sub_proof_request();
        _process_credential_signature(credential_signature,
                                      signature_correctness_proof,
                                      master_secret_blinding_data,
                                      master_secret,
                                      credential_pub_key,
                                      credential_issuance_nonce,
                                      ptr::null(),
                                      ptr::null(),
                                      ptr::null());

        let proof_building_nonce = _nonce();
        let proof = _proof(credential_pub_key,
                           credential_signature,
                           proof_building_nonce,
                           master_secret,
                           ptr::null(),
                           ptr::null());

        let proof_verifier = _proof_verifier();
        _add_sub_proof_request(proof_verifier, key_id, credential_schema, credential_pub_key, sub_proof_request, ptr::null(), ptr::null());

        let mut valid = false;
        let err_code = indy_crypto_cl_proof_verifier_verify(proof_verifier, proof, proof_building_nonce, &mut valid);
        assert_eq!(err_code, ErrorCode::Success);
        assert!(valid);

        _free_credential_def(credential_pub_key, credential_priv_key, credential_key_correctness_proof);
        _free_master_secret(master_secret);
        _free_blinded_master_secret(blinded_master_secret, master_secret_blinding_data, blinded_master_secret_correctness_proof);
        _free_nonce(master_secret_blinding_nonce);
        _free_nonce(credential_issuance_nonce);
        _free_nonce(proof_building_nonce);
        _free_credential_schema(credential_schema);
        _free_sub_proof_request(sub_proof_request);
        _free_credential_signature(credential_signature, signature_correctness_proof);
    }

    #[test]
    fn indy_crypto_cl_proof_verifier_verify_works_for_revocation_proof() {
        let key_id = CString::new("key_id").unwrap();
        let (credential_pub_key, credential_priv_key, credential_key_correctness_proof) = _credential_def();
        let (rev_key_pub, rev_key_priv, rev_reg, rev_tails_generator) = _revocation_registry_def(credential_pub_key);
        let master_secret = _master_secret();
        let master_secret_blinding_nonce = _nonce();
        let (blinded_master_secret, master_secret_blinding_data,
            blinded_master_secret_correctness_proof) = _blinded_master_secret(credential_pub_key,
                                                                              credential_key_correctness_proof,
                                                                              master_secret,
                                                                              master_secret_blinding_nonce);
        let credential_issuance_nonce = _nonce();
        let tail_storage = FFISimpleTailStorage::new(rev_tails_generator);

        let (credential_signature, signature_correctness_proof, rev_reg_delta) =
            _credential_signature_with_revoc(blinded_master_secret,
                                             blinded_master_secret_correctness_proof,
                                             master_secret_blinding_nonce,
                                             credential_issuance_nonce,
                                             credential_pub_key,
                                             credential_priv_key,
                                             rev_key_priv,
                                             rev_reg,
                                             tail_storage.get_ctx());
        let credential_schema = _credential_schema();
        let sub_proof_request = _sub_proof_request();
        let witness = _witness(rev_reg_delta, tail_storage.get_ctx());
        _process_credential_signature(credential_signature,
                                      signature_correctness_proof,
                                      master_secret_blinding_data,
                                      master_secret,
                                      credential_pub_key,
                                      credential_issuance_nonce,
                                      rev_key_pub,
                                      rev_reg,
                                      witness);

        let proof_building_nonce = _nonce();
        let proof = _proof(credential_pub_key,
                           credential_signature,
                           proof_building_nonce,
                           master_secret,
                           rev_reg,
                           witness);

        let proof_verifier = _proof_verifier();
        _add_sub_proof_request(proof_verifier, key_id, credential_schema, credential_pub_key, sub_proof_request, rev_key_pub, rev_reg);

        let mut valid = false;
        let err_code = indy_crypto_cl_proof_verifier_verify(proof_verifier, proof, proof_building_nonce, &mut valid);
        assert_eq!(err_code, ErrorCode::Success);
        assert!(valid);

        _free_credential_def(credential_pub_key, credential_priv_key, credential_key_correctness_proof);
        _free_master_secret(master_secret);
        _free_blinded_master_secret(blinded_master_secret, master_secret_blinding_data, blinded_master_secret_correctness_proof);
        _free_nonce(master_secret_blinding_nonce);
        _free_nonce(credential_issuance_nonce);
        _free_nonce(proof_building_nonce);
        _free_witness(witness);
        _free_credential_schema(credential_schema);
        _free_sub_proof_request(sub_proof_request);
        _free_credential_signature(credential_signature, signature_correctness_proof);
    }
}

pub mod mocks {
    use super::*;
    use std::ptr;
    use std::ffi::CString;

    pub fn _proof_verifier() -> *const c_void {
        let mut proof_verifier_p: *const c_void = ptr::null();
        let err_code = indy_crypto_cl_verifier_new_proof_verifier(&mut proof_verifier_p);
        assert_eq!(err_code, ErrorCode::Success);
        assert!(!proof_verifier_p.is_null());

        proof_verifier_p
    }

    pub fn _add_sub_proof_request(proof_verifier: *const c_void, key_id: CString, credential_schema: *const c_void,
                                  credential_pub_key: *const c_void, sub_proof_request: *const c_void, rev_key_pub: *const c_void, rev_reg: *const c_void) {
        let err_code = indy_crypto_cl_proof_verifier_add_sub_proof_request(proof_verifier,
                                                                           key_id.as_ptr(),
                                                                           sub_proof_request,
                                                                           credential_schema,
                                                                           credential_pub_key,
                                                                           rev_key_pub,
                                                                           rev_reg);
        assert_eq!(err_code, ErrorCode::Success);
    }

    pub fn _free_proof_verifier(proof_verifier: *const c_void, proof: *const c_void, nonce: *const c_void) {
        let mut valid = false;
        let err_code = indy_crypto_cl_proof_verifier_verify(proof_verifier, proof, nonce, &mut valid);
        assert_eq!(err_code, ErrorCode::Success);
    }
}