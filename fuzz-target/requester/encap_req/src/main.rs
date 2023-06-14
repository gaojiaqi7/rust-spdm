// Copyright (c) 2023 Intel Corporation
//
// SPDX-License-Identifier: BSD-2-Clause-Patent

use fuzzlib::spdmlib::message::SpdmKeyExchangeMutAuthAttributes;
use fuzzlib::*;
use spdmlib::common::session::{SpdmSession, SpdmSessionState};
use spdmlib::common::SpdmConnectionState;
use spdmlib::protocol::*;

fn fuzz_session_based_mutual_authenticate(fuzzdata: &[u8]) {
    spdmlib::secret::asym_sign::register(SECRET_ASYM_IMPL_INSTANCE.clone());
    spdmlib::crypto::aead::register(FAKE_AEAD.clone());
    // TCD:
    // - id: 0
    // - title: 'Fuzz SPDM handle start session based mutual authenticate'
    // - description: '<p>Requester start mutual authenticate without using the encapsulated request flow.</p>'
    // -
    {
        let (req_config_info, req_provision_info) = req_create_info();
        let shared_buffer = SharedBuffer::new();
        let pcidoe_transport_encap = &mut PciDoeTransportEncap {};
        let mut device_io_requester = fake_device_io::FakeSpdmDeviceIo::new(&shared_buffer);
        device_io_requester.set_rx(fuzzdata);

        let mut requester = requester::RequesterContext::new(
            &mut device_io_requester,
            pcidoe_transport_encap,
            req_config_info,
            req_provision_info,
        );

        requester.common.negotiate_info.spdm_version_sel = SpdmVersion::SpdmVersion12;
        requester.common.negotiate_info.base_hash_sel = SpdmBaseHashAlgo::TPM_ALG_SHA_384;

        requester.common.negotiate_info.req_capabilities_sel =
            requester.common.negotiate_info.req_capabilities_sel
                | SpdmRequestCapabilityFlags::ENCAP_CAP
                | SpdmRequestCapabilityFlags::CERT_CAP;
        requester.common.negotiate_info.rsp_capabilities_sel =
            requester.common.negotiate_info.rsp_capabilities_sel
                | SpdmResponseCapabilityFlags::ENCAP_CAP;

        requester.common.session[0] = SpdmSession::new();
        requester.common.session[0].setup(4294836221).unwrap();
        requester.common.session[0].set_session_state(SpdmSessionState::SpdmSessionHandshaking);
        requester.common.session[0].set_crypto_param(
            SpdmBaseHashAlgo::TPM_ALG_SHA_384,
            SpdmDheAlgo::SECP_384_R1,
            SpdmAeadAlgo::AES_256_GCM,
            SpdmKeyScheduleAlgo::SPDM_KEY_SCHEDULE,
        );
        requester.common.session[0]
            .set_mut_auth_requested(SpdmKeyExchangeMutAuthAttributes::MUT_AUTH_REQ);

        let _ = requester
            .session_based_mutual_authenticate(4294836221)
            .is_ok();
    }
    // TCD:
    // - id: 0
    // - title: 'Fuzz SPDM handle start session based mutual authenticate'
    // - description: '<p>Requester start mutual authenticate with the encapsulated request flow (not optimized).</p>'
    // -
    {
        let (req_config_info, req_provision_info) = req_create_info();
        let shared_buffer = SharedBuffer::new();
        let pcidoe_transport_encap = &mut PciDoeTransportEncap {};
        let mut device_io_requester = fake_device_io::FakeSpdmDeviceIo::new(&shared_buffer);
        device_io_requester.set_rx(fuzzdata);

        let mut requester = requester::RequesterContext::new(
            &mut device_io_requester,
            pcidoe_transport_encap,
            req_config_info,
            req_provision_info,
        );

        requester.common.negotiate_info.spdm_version_sel = SpdmVersion::SpdmVersion12;
        requester.common.negotiate_info.base_hash_sel = SpdmBaseHashAlgo::TPM_ALG_SHA_384;

        requester.common.negotiate_info.req_capabilities_sel =
            requester.common.negotiate_info.req_capabilities_sel
                | SpdmRequestCapabilityFlags::ENCAP_CAP
                | SpdmRequestCapabilityFlags::CERT_CAP;
        requester.common.negotiate_info.rsp_capabilities_sel =
            requester.common.negotiate_info.rsp_capabilities_sel
                | SpdmResponseCapabilityFlags::ENCAP_CAP;

        requester.common.session[0] = SpdmSession::new();
        requester.common.session[0].setup(4294836221).unwrap();
        requester.common.session[0].set_session_state(SpdmSessionState::SpdmSessionHandshaking);
        requester.common.session[0].set_crypto_param(
            SpdmBaseHashAlgo::TPM_ALG_SHA_384,
            SpdmDheAlgo::SECP_384_R1,
            SpdmAeadAlgo::AES_256_GCM,
            SpdmKeyScheduleAlgo::SPDM_KEY_SCHEDULE,
        );
        requester.common.session[0].set_mut_auth_requested(
            SpdmKeyExchangeMutAuthAttributes::MUT_AUTH_REQ_WITH_ENCAP_REQUEST,
        );

        let _ = requester
            .session_based_mutual_authenticate(4294836221)
            .is_ok();
    }
    // TCD:
    // - id: 0
    // - title: 'Fuzz SPDM handle start session based mutual authenticate'
    // - description: '<p>Requester start mutual authenticate with the optimized encapsulated request flow.</p>'
    // -
    {
        let (req_config_info, req_provision_info) = req_create_info();
        let shared_buffer = SharedBuffer::new();
        let pcidoe_transport_encap = &mut PciDoeTransportEncap {};
        let mut device_io_requester = fake_device_io::FakeSpdmDeviceIo::new(&shared_buffer);
        device_io_requester.set_rx(fuzzdata);

        let mut requester = requester::RequesterContext::new(
            &mut device_io_requester,
            pcidoe_transport_encap,
            req_config_info,
            req_provision_info,
        );

        requester.common.negotiate_info.spdm_version_sel = SpdmVersion::SpdmVersion12;
        requester.common.negotiate_info.base_hash_sel = SpdmBaseHashAlgo::TPM_ALG_SHA_384;

        requester.common.negotiate_info.req_capabilities_sel =
            requester.common.negotiate_info.req_capabilities_sel
                | SpdmRequestCapabilityFlags::ENCAP_CAP
                | SpdmRequestCapabilityFlags::CERT_CAP;
        requester.common.negotiate_info.rsp_capabilities_sel =
            requester.common.negotiate_info.rsp_capabilities_sel
                | SpdmResponseCapabilityFlags::ENCAP_CAP;

        requester.common.session[0] = SpdmSession::new();
        requester.common.session[0].setup(4294836221).unwrap();
        requester.common.session[0].set_session_state(SpdmSessionState::SpdmSessionHandshaking);
        requester.common.session[0].set_crypto_param(
            SpdmBaseHashAlgo::TPM_ALG_SHA_384,
            SpdmDheAlgo::SECP_384_R1,
            SpdmAeadAlgo::AES_256_GCM,
            SpdmKeyScheduleAlgo::SPDM_KEY_SCHEDULE,
        );
        requester.common.session[0].set_mut_auth_requested(
            SpdmKeyExchangeMutAuthAttributes::MUT_AUTH_REQ_WITH_GET_DIGESTS,
        );

        let _ = requester
            .session_based_mutual_authenticate(4294836221)
            .is_ok();
    }
    // TCD:
    // - id: 0
    // - title: 'Fuzz SPDM handle start session based mutual authenticate'
    // - description: '<p>Requester start mutual authenticate failed due to no mut_auth_requested.</p>'
    // -
    {
        let (req_config_info, req_provision_info) = req_create_info();
        let shared_buffer = SharedBuffer::new();
        let pcidoe_transport_encap = &mut PciDoeTransportEncap {};
        let mut device_io_requester = fake_device_io::FakeSpdmDeviceIo::new(&shared_buffer);
        device_io_requester.set_rx(fuzzdata);

        let mut requester = requester::RequesterContext::new(
            &mut device_io_requester,
            pcidoe_transport_encap,
            req_config_info,
            req_provision_info,
        );

        requester.common.negotiate_info.spdm_version_sel = SpdmVersion::SpdmVersion12;
        requester.common.negotiate_info.base_hash_sel = SpdmBaseHashAlgo::TPM_ALG_SHA_384;

        requester.common.negotiate_info.req_capabilities_sel =
            requester.common.negotiate_info.req_capabilities_sel
                | SpdmRequestCapabilityFlags::ENCAP_CAP
                | SpdmRequestCapabilityFlags::CERT_CAP;
        requester.common.negotiate_info.rsp_capabilities_sel =
            requester.common.negotiate_info.rsp_capabilities_sel
                | SpdmResponseCapabilityFlags::ENCAP_CAP;

        requester.common.session[0] = SpdmSession::new();
        requester.common.session[0].setup(4294836221).unwrap();
        requester.common.session[0].set_session_state(SpdmSessionState::SpdmSessionHandshaking);
        requester.common.session[0].set_crypto_param(
            SpdmBaseHashAlgo::TPM_ALG_SHA_384,
            SpdmDheAlgo::SECP_384_R1,
            SpdmAeadAlgo::AES_256_GCM,
            SpdmKeyScheduleAlgo::SPDM_KEY_SCHEDULE,
        );

        let _ = requester
            .session_based_mutual_authenticate(4294836221)
            .is_ok();
    }
}

#[cfg(not(feature = "use_libfuzzer"))]
fn main() {
    #[cfg(all(feature = "fuzzlogfile", feature = "fuzz"))]
    flexi_logger::Logger::try_with_str("info")
        .unwrap()
        .log_to_file(
            FileSpec::default()
                .directory("traces")
                .basename("foo")
                .discriminant("Sample4711A")
                .suffix("trc"),
        )
        .print_message()
        .create_symlink("current_run")
        .start()
        .unwrap();

    #[cfg(not(feature = "fuzz"))]
    {
        let args: Vec<String> = std::env::args().collect();
        if args.len() < 2 {
            // Here you can replace the single-step debugging value in the fuzzdata array.
            let fuzzdata = include_bytes!("../../../in/encap_req/encap_resp_ack.raw");
            fuzz_session_based_mutual_authenticate(fuzzdata);
        } else {
            let path = &args[1];
            let data = std::fs::read(path).expect("read crash file fail");
            fuzz_session_based_mutual_authenticate(data.as_slice());
        }
    }
    #[cfg(feature = "fuzz")]
    afl::fuzz!(|data: &[u8]| {
        fuzz_session_based_mutual_authenticate(data);
    });
}
