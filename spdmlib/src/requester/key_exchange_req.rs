// Copyright (c) 2020 Intel Corporation
//
// SPDX-License-Identifier: BSD-2-Clause-Patent

extern crate alloc;
use alloc::boxed::Box;

use crate::common::session::SpdmSession;
use crate::error::SPDM_STATUS_BUFFER_FULL;
use crate::error::SPDM_STATUS_CRYPTO_ERROR;
use crate::error::SPDM_STATUS_ERROR_PEER;
use crate::error::SPDM_STATUS_INVALID_MSG_FIELD;
use crate::error::SPDM_STATUS_INVALID_PARAMETER;
#[cfg(feature = "hashed-transcript-data")]
use crate::error::SPDM_STATUS_INVALID_STATE_LOCAL;
use crate::error::SPDM_STATUS_SESSION_NUMBER_EXCEED;
use crate::error::SPDM_STATUS_UNSUPPORTED_CAP;
use crate::error::SPDM_STATUS_VERIF_FAIL;
use crate::protocol::*;
use crate::requester::*;

use crate::crypto;

use crate::error::SpdmResult;
use crate::message::*;
use crate::protocol::{SpdmMeasurementSummaryHashType, SpdmSignatureStruct, SpdmVersion};

impl<'a> RequesterContext<'a> {
    pub fn send_receive_spdm_key_exchange(
        &mut self,
        slot_id: u8,
        measurement_summary_hash_type: SpdmMeasurementSummaryHashType,
    ) -> SpdmResult<u32> {
        info!("send spdm key exchange\n");

        if slot_id >= SPDM_MAX_SLOT_NUMBER as u8 {
            return Err(SPDM_STATUS_INVALID_PARAMETER);
        }

        let req_session_id = self.common.get_next_half_session_id(true)?;

        self.common
            .reset_buffer_via_request_code(SpdmRequestResponseCode::SpdmRequestKeyExchange, None);

        let mut send_buffer = [0u8; config::MAX_SPDM_MSG_SIZE];
        let (key_exchange_context, send_used) = self.encode_spdm_key_exchange(
            req_session_id,
            &mut send_buffer,
            slot_id,
            measurement_summary_hash_type,
        )?;
        self.send_message(&send_buffer[..send_used])?;

        // Receive
        let mut receive_buffer = [0u8; config::MAX_SPDM_MSG_SIZE];
        let receive_used = self.receive_message(&mut receive_buffer, false)?;
        self.handle_spdm_key_exhcange_response(
            req_session_id,
            slot_id,
            &send_buffer[..send_used],
            &receive_buffer[..receive_used],
            measurement_summary_hash_type,
            key_exchange_context,
        )
    }

    pub fn encode_spdm_key_exchange(
        &mut self,
        req_session_id: u16,
        buf: &mut [u8],
        slot_id: u8,
        measurement_summary_hash_type: SpdmMeasurementSummaryHashType,
    ) -> SpdmResult<(Box<dyn crypto::SpdmDheKeyExchange>, usize)> {
        let mut writer = Writer::init(buf);

        let mut random = [0u8; SPDM_RANDOM_SIZE];
        crypto::rand::get_random(&mut random)?;

        let (exchange, key_exchange_context) =
            crypto::dhe::generate_key_pair(self.common.negotiate_info.dhe_sel)
                .ok_or(SPDM_STATUS_CRYPTO_ERROR)?;

        debug!("!!! exchange data : {:02x?}\n", exchange);

        let mut opaque;
        if self.common.negotiate_info.spdm_version_sel.get_u8()
            < SpdmVersion::SpdmVersion12.get_u8()
        {
            opaque = SpdmOpaqueStruct {
                data_size: crate::common::opaque::REQ_DMTF_OPAQUE_DATA_SUPPORT_VERSION_LIST_DSP0277
                    .len() as u16,
                ..Default::default()
            };
            opaque.data[..(opaque.data_size as usize)].copy_from_slice(
                crate::common::opaque::REQ_DMTF_OPAQUE_DATA_SUPPORT_VERSION_LIST_DSP0277.as_ref(),
            );
        } else if self.common.negotiate_info.opaque_data_support
            == SpdmOpaqueSupport::OPAQUE_DATA_FMT1
        {
            opaque = SpdmOpaqueStruct {
                data_size:
                    crate::common::opaque::REQ_DMTF_OPAQUE_DATA_SUPPORT_VERSION_LIST_DSP0274_FMT1
                        .len() as u16,
                ..Default::default()
            };
            opaque.data[..(opaque.data_size as usize)].copy_from_slice(
                crate::common::opaque::REQ_DMTF_OPAQUE_DATA_SUPPORT_VERSION_LIST_DSP0274_FMT1
                    .as_ref(),
            );
        } else {
            return Err(SPDM_STATUS_UNSUPPORTED_CAP);
        }

        let request = SpdmMessage {
            header: SpdmMessageHeader {
                version: self.common.negotiate_info.spdm_version_sel,
                request_response_code: SpdmRequestResponseCode::SpdmRequestKeyExchange,
            },
            payload: SpdmMessagePayload::SpdmKeyExchangeRequest(SpdmKeyExchangeRequestPayload {
                slot_id,
                measurement_summary_hash_type,
                req_session_id,
                session_policy: self.common.config_info.session_policy,
                random: SpdmRandomStruct { data: random },
                exchange,
                opaque,
            }),
        };
        request.spdm_encode(&mut self.common, &mut writer)?;
        Ok((key_exchange_context, writer.used()))
    }

    pub fn handle_spdm_key_exhcange_response(
        &mut self,
        req_session_id: u16,
        slot_id: u8,
        send_buffer: &[u8],
        receive_buffer: &[u8],
        measurement_summary_hash_type: SpdmMeasurementSummaryHashType,
        key_exchange_context: Box<dyn crypto::SpdmDheKeyExchange>,
    ) -> SpdmResult<u32> {
        if (measurement_summary_hash_type
            == SpdmMeasurementSummaryHashType::SpdmMeasurementSummaryHashTypeTcb)
            || (measurement_summary_hash_type
                == SpdmMeasurementSummaryHashType::SpdmMeasurementSummaryHashTypeAll)
        {
            self.common.runtime_info.need_measurement_summary_hash = true;
        } else {
            self.common.runtime_info.need_measurement_summary_hash = false;
        }

        let in_clear_text = self
            .common
            .negotiate_info
            .req_capabilities_sel
            .contains(SpdmRequestCapabilityFlags::HANDSHAKE_IN_THE_CLEAR_CAP)
            && self
                .common
                .negotiate_info
                .rsp_capabilities_sel
                .contains(SpdmResponseCapabilityFlags::HANDSHAKE_IN_THE_CLEAR_CAP);
        info!("in_clear_text {:?}\n", in_clear_text);

        let mut reader = Reader::init(receive_buffer);
        match SpdmMessageHeader::read(&mut reader) {
            Some(message_header) => {
                if message_header.version != self.common.negotiate_info.spdm_version_sel {
                    return Err(SPDM_STATUS_INVALID_MSG_FIELD);
                }
                match message_header.request_response_code {
                    SpdmRequestResponseCode::SpdmResponseKeyExchangeRsp => {
                        let key_exchange_rsp = SpdmKeyExchangeResponsePayload::spdm_read(
                            &mut self.common,
                            &mut reader,
                        );
                        let receive_used = reader.used();
                        if let Some(key_exchange_rsp) = key_exchange_rsp {
                            debug!("!!! key_exchange rsp : {:02x?}\n", key_exchange_rsp);
                            debug!(
                                "!!! exchange data (peer) : {:02x?}\n",
                                &key_exchange_rsp.exchange
                            );

                            let final_key = key_exchange_context
                                .compute_final_key(&key_exchange_rsp.exchange)
                                .ok_or(SPDM_STATUS_CRYPTO_ERROR)?;

                            debug!("!!! final_key : {:02x?}\n", final_key.as_ref());

                            // create session structure
                            let base_hash_algo = self.common.negotiate_info.base_hash_sel;
                            let dhe_algo = self.common.negotiate_info.dhe_sel;
                            let aead_algo = self.common.negotiate_info.aead_sel;
                            let key_schedule_algo = self.common.negotiate_info.key_schedule_sel;
                            let sequence_number_count =
                                self.common.transport_encap.get_sequence_number_count();
                            let max_random_count =
                                self.common.transport_encap.get_max_random_count();

                            let secure_spdm_version_sel = if let Some(secured_message_version) =
                                key_exchange_rsp
                                    .opaque
                                    .req_get_dmtf_secure_spdm_version_selection(&mut self.common)
                            {
                                secured_message_version.get_secure_spdm_version()
                            } else {
                                0
                            };

                            info!(
                                "secure_spdm_version_sel set to {:02X?}",
                                secure_spdm_version_sel
                            );

                            let session_id = ((key_exchange_rsp.rsp_session_id as u32) << 16)
                                + req_session_id as u32;
                            let spdm_version_sel = self.common.negotiate_info.spdm_version_sel;
                            let message_a = self.common.runtime_info.message_a.clone();
                            let cert_chain_hash =
                                self.common.get_certchain_hash_peer(false, slot_id as usize);
                            if cert_chain_hash.is_none() {
                                return Err(SPDM_STATUS_INVALID_MSG_FIELD);
                            }

                            #[cfg(feature = "mut-auth")]
                            if !key_exchange_rsp.mut_auth_req.is_empty() {
                                if !self
                                    .common
                                    .negotiate_info
                                    .req_capabilities_sel
                                    .contains(SpdmRequestCapabilityFlags::MUT_AUTH_CAP)
                                    || !self
                                        .common
                                        .negotiate_info
                                        .rsp_capabilities_sel
                                        .contains(SpdmResponseCapabilityFlags::MUT_AUTH_CAP)
                                {
                                    return Err(SPDM_STATUS_INVALID_MSG_FIELD);
                                }
                                if key_exchange_rsp.mut_auth_req
                                    == SpdmKeyExchangeMutAuthAttributes::MUT_AUTH_REQ_WITH_ENCAP_REQUEST
                                    && key_exchange_rsp.req_slot_id >= SPDM_MAX_SLOT_NUMBER as u8
                                {
                                    return Err(SPDM_STATUS_INVALID_MSG_FIELD);
                                }
                                self.common.runtime_info.set_local_used_cert_chain_slot_id(
                                    key_exchange_rsp.req_slot_id & 0xf,
                                );
                            }

                            let session = self
                                .common
                                .get_next_avaiable_session()
                                .ok_or(SPDM_STATUS_SESSION_NUMBER_EXCEED)?;

                            session.setup(session_id)?;

                            session.set_use_psk(false);
                            session.set_mut_auth_requested(key_exchange_rsp.mut_auth_req);

                            session.set_crypto_param(
                                base_hash_algo,
                                dhe_algo,
                                aead_algo,
                                key_schedule_algo,
                            );
                            session.set_transport_param(sequence_number_count, max_random_count);
                            session.set_dhe_secret(spdm_version_sel, final_key)?;
                            session.runtime_info.message_a = message_a;
                            session.runtime_info.rsp_cert_hash = cert_chain_hash;
                            session.runtime_info.req_cert_hash = None;

                            // create transcript
                            let base_asym_size =
                                self.common.negotiate_info.base_asym_sel.get_size() as usize;
                            let base_hash_size =
                                self.common.negotiate_info.base_hash_sel.get_size() as usize;
                            let temp_receive_used = if in_clear_text {
                                receive_used - base_asym_size
                            } else {
                                receive_used - base_asym_size - base_hash_size
                            };

                            self.common.append_message_k(session_id, send_buffer)?;
                            self.common.append_message_k(
                                session_id,
                                &receive_buffer[..temp_receive_used],
                            )?;

                            let session = self
                                .common
                                .get_immutable_session_via_id(session_id)
                                .unwrap();

                            // verify signature
                            if self
                                .verify_key_exchange_rsp_signature(
                                    slot_id,
                                    session,
                                    &key_exchange_rsp.signature,
                                )
                                .is_err()
                            {
                                error!("verify_key_exchange_rsp_signature fail");
                                return Err(SPDM_STATUS_VERIF_FAIL);
                            } else {
                                info!("verify_key_exchange_rsp_signature pass");
                            }

                            self.common.append_message_k(
                                session_id,
                                key_exchange_rsp.signature.as_ref(),
                            )?;

                            let session = self
                                .common
                                .get_immutable_session_via_id(session_id)
                                .unwrap();

                            // generate the handshake secret (including finished_key) before verify HMAC
                            let th1 = self
                                .common
                                .calc_req_transcript_hash(false, slot_id, false, session)?;
                            debug!("!!! th1 : {:02x?}\n", th1.as_ref());

                            let session = self.common.get_session_via_id(session_id).unwrap();
                            session.generate_handshake_secret(spdm_version_sel, &th1)?;

                            if !in_clear_text {
                                let session = self
                                    .common
                                    .get_immutable_session_via_id(session_id)
                                    .unwrap();

                                // verify HMAC with finished_key
                                let transcript_hash = self
                                    .common
                                    .calc_req_transcript_hash(false, slot_id, false, session)?;

                                let session = self
                                    .common
                                    .get_immutable_session_via_id(session_id)
                                    .unwrap();

                                if session
                                    .verify_hmac_with_response_finished_key(
                                        transcript_hash.as_ref(),
                                        &key_exchange_rsp.verify_data,
                                    )
                                    .is_err()
                                {
                                    error!("verify_hmac_with_response_finished_key fail");
                                    let session =
                                        self.common.get_session_via_id(session_id).unwrap();
                                    let _ = session.teardown(session_id);
                                    return Err(SPDM_STATUS_VERIF_FAIL);
                                } else {
                                    info!("verify_hmac_with_response_finished_key pass");
                                }

                                // append verify_data after TH1
                                if self
                                    .common
                                    .append_message_k(
                                        session_id,
                                        key_exchange_rsp.verify_data.as_ref(),
                                    )
                                    .is_err()
                                {
                                    let session =
                                        self.common.get_session_via_id(session_id).unwrap();
                                    let _ = session.teardown(session_id);
                                    return Err(SPDM_STATUS_BUFFER_FULL);
                                }
                            }

                            // append verify_data after TH1
                            let session = self.common.get_session_via_id(session_id).unwrap();

                            session.secure_spdm_version_sel = secure_spdm_version_sel;
                            session.heartbeat_period = key_exchange_rsp.heartbeat_period;

                            session.set_session_state(
                                crate::common::session::SpdmSessionState::SpdmSessionHandshaking,
                            );

                            if in_clear_text {
                                self.common
                                    .runtime_info
                                    .set_last_session_id(Some(session_id));
                            }

                            Ok(session_id)
                        } else {
                            error!("!!! key_exchange : fail !!!\n");
                            Err(SPDM_STATUS_INVALID_MSG_FIELD)
                        }
                    }
                    SpdmRequestResponseCode::SpdmResponseError => {
                        let status = self.spdm_handle_error_response_main(
                            None,
                            receive_buffer,
                            SpdmRequestResponseCode::SpdmRequestKeyExchange,
                            SpdmRequestResponseCode::SpdmResponseKeyExchangeRsp,
                        );
                        match status {
                            Err(status) => Err(status),
                            Ok(()) => Err(SPDM_STATUS_ERROR_PEER),
                        }
                    }
                    _ => Err(SPDM_STATUS_ERROR_PEER),
                }
            }
            None => Err(SPDM_STATUS_INVALID_MSG_FIELD),
        }
    }

    #[cfg(feature = "hashed-transcript-data")]
    pub fn verify_key_exchange_rsp_signature(
        &self,
        slot_id: u8,
        session: &SpdmSession,
        signature: &SpdmSignatureStruct,
    ) -> SpdmResult {
        let transcript_hash = self
            .common
            .calc_req_transcript_hash(false, slot_id, false, session)?;

        debug!("message_hash - {:02x?}", transcript_hash.as_ref());

        if self.common.peer_info.peer_cert_chain[slot_id as usize].is_none() {
            error!("peer_cert_chain is not populated!\n");
            return Err(SPDM_STATUS_INVALID_PARAMETER);
        }

        let cert_chain_data = &self.common.peer_info.peer_cert_chain[slot_id as usize]
            .as_ref()
            .ok_or(SPDM_STATUS_INVALID_PARAMETER)?
            .data[(4usize + self.common.negotiate_info.base_hash_sel.get_size() as usize)
            ..(self.common.peer_info.peer_cert_chain[slot_id as usize]
                .as_ref()
                .ok_or(SPDM_STATUS_INVALID_PARAMETER)?
                .data_size as usize)];

        let mut message_sign = ManagedBuffer12Sign::default();
        if self.common.negotiate_info.spdm_version_sel.get_u8()
            >= SpdmVersion::SpdmVersion12.get_u8()
        {
            message_sign.reset_message();
            message_sign
                .append_message(&SPDM_VERSION_1_2_SIGNING_PREFIX_CONTEXT)
                .ok_or(SPDM_STATUS_BUFFER_FULL)?;
            message_sign
                .append_message(&SPDM_VERSION_1_2_SIGNING_CONTEXT_ZEROPAD_2)
                .ok_or(SPDM_STATUS_BUFFER_FULL)?;
            message_sign
                .append_message(&SPDM_KEY_EXCHANGE_RESPONSE_SIGN_CONTEXT)
                .ok_or(SPDM_STATUS_BUFFER_FULL)?;
            message_sign
                .append_message(transcript_hash.as_ref())
                .ok_or(SPDM_STATUS_BUFFER_FULL)?;
        } else {
            error!("hashed-transcript-data is unsupported in SPDM 1.0/1.1 signing verification!\n");
            return Err(SPDM_STATUS_INVALID_STATE_LOCAL);
        }

        crypto::asym_verify::verify(
            self.common.negotiate_info.base_hash_sel,
            self.common.negotiate_info.base_asym_sel,
            cert_chain_data,
            message_sign.as_ref(),
            signature,
        )
    }

    #[cfg(not(feature = "hashed-transcript-data"))]
    pub fn verify_key_exchange_rsp_signature(
        &self,
        slot_id: u8,
        session: &SpdmSession,
        signature: &SpdmSignatureStruct,
    ) -> SpdmResult {
        let message_hash = self
            .common
            .calc_req_transcript_hash(false, slot_id, false, session)?;
        // we dont need create message hash for verify
        // we just print message hash for debug purpose
        debug!("message_hash - {:02x?}", message_hash.as_ref());

        if self.common.peer_info.peer_cert_chain[slot_id as usize].is_none() {
            error!("peer_cert_chain is not populated!\n");
            return Err(SPDM_STATUS_INVALID_PARAMETER);
        }

        let cert_chain_data = &self.common.peer_info.peer_cert_chain[slot_id as usize]
            .as_ref()
            .ok_or(SPDM_STATUS_INVALID_PARAMETER)?
            .data[(4usize + self.common.negotiate_info.base_hash_sel.get_size() as usize)
            ..(self.common.peer_info.peer_cert_chain[slot_id as usize]
                .as_ref()
                .ok_or(SPDM_STATUS_INVALID_PARAMETER)?
                .data_size as usize)];

        let mut message = self.common.calc_req_transcript_data(
            false,
            slot_id,
            false,
            &session.runtime_info.message_k,
            None,
        )?;

        if self.common.negotiate_info.spdm_version_sel.get_u8()
            >= SpdmVersion::SpdmVersion12.get_u8()
        {
            message.reset_message();
            message
                .append_message(&SPDM_VERSION_1_2_SIGNING_PREFIX_CONTEXT)
                .ok_or(SPDM_STATUS_BUFFER_FULL)?;
            message
                .append_message(&SPDM_VERSION_1_2_SIGNING_CONTEXT_ZEROPAD_2)
                .ok_or(SPDM_STATUS_BUFFER_FULL)?;
            message
                .append_message(&SPDM_KEY_EXCHANGE_RESPONSE_SIGN_CONTEXT)
                .ok_or(SPDM_STATUS_BUFFER_FULL)?;
            message
                .append_message(message_hash.as_ref())
                .ok_or(SPDM_STATUS_BUFFER_FULL)?;
        }

        crypto::asym_verify::verify(
            self.common.negotiate_info.base_hash_sel,
            self.common.negotiate_info.base_asym_sel,
            cert_chain_data,
            message.as_ref(),
            signature,
        )
    }
}

#[cfg(all(test,))]
mod tests_requester {
    use super::*;
    use crate::responder;
    use crate::testlib::*;

    #[test]
    fn test_case0_send_receive_spdm_key_exchange() {
        let (rsp_config_info, rsp_provision_info) = create_info();
        let (req_config_info, req_provision_info) = create_info();

        let shared_buffer = SharedBuffer::new();
        let mut device_io_responder = FakeSpdmDeviceIoReceve::new(&shared_buffer);
        let pcidoe_transport_encap = &mut PciDoeTransportEncap {};

        crate::secret::asym_sign::register(SECRET_ASYM_IMPL_INSTANCE.clone());

        let mut responder = responder::ResponderContext::new(
            &mut device_io_responder,
            pcidoe_transport_encap,
            rsp_config_info,
            rsp_provision_info,
        );

        responder.common.provision_info.my_cert_chain = [
            Some(get_rsp_cert_chain_buff()),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ];

        responder.common.negotiate_info.base_hash_sel = SpdmBaseHashAlgo::TPM_ALG_SHA_384;
        responder.common.negotiate_info.aead_sel = SpdmAeadAlgo::AES_128_GCM;
        responder.common.negotiate_info.dhe_sel = SpdmDheAlgo::SECP_384_R1;
        responder.common.negotiate_info.base_asym_sel =
            SpdmBaseAsymAlgo::TPM_ALG_ECDSA_ECC_NIST_P384;
        responder.common.negotiate_info.opaque_data_support = SpdmOpaqueSupport::OPAQUE_DATA_FMT1;

        responder.common.reset_runtime_info();

        responder.common.provision_info.my_cert_chain = [
            Some(get_rsp_cert_chain_buff()),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ];
        responder.common.negotiate_info.spdm_version_sel = SpdmVersion::SpdmVersion12;
        responder
            .common
            .runtime_info
            .set_connection_state(SpdmConnectionState::SpdmConnectionNegotiated);

        let pcidoe_transport_encap2 = &mut PciDoeTransportEncap {};
        let mut device_io_requester = FakeSpdmDeviceIo::new(&shared_buffer, &mut responder);

        let mut requester = RequesterContext::new(
            &mut device_io_requester,
            pcidoe_transport_encap2,
            req_config_info,
            req_provision_info,
        );

        requester.common.negotiate_info.base_hash_sel = SpdmBaseHashAlgo::TPM_ALG_SHA_384;
        requester.common.negotiate_info.aead_sel = SpdmAeadAlgo::AES_128_GCM;
        requester.common.negotiate_info.dhe_sel = SpdmDheAlgo::SECP_384_R1;
        requester.common.negotiate_info.base_asym_sel =
            SpdmBaseAsymAlgo::TPM_ALG_ECDSA_ECC_NIST_P384;
        requester.common.negotiate_info.opaque_data_support = SpdmOpaqueSupport::OPAQUE_DATA_FMT1;
        requester.common.negotiate_info.spdm_version_sel = SpdmVersion::SpdmVersion12;

        requester.common.reset_runtime_info();

        requester.common.peer_info.peer_cert_chain[0] = Some(get_rsp_cert_chain_buff());

        let measurement_summary_hash_type =
            SpdmMeasurementSummaryHashType::SpdmMeasurementSummaryHashTypeNone;
        let status = requester
            .send_receive_spdm_key_exchange(0, measurement_summary_hash_type)
            .is_ok();
        assert!(status);
    }
}
