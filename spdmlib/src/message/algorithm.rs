// Copyright (c) 2020 Intel Corporation
//
// SPDX-License-Identifier: BSD-2-Clause-Patent

pub use crate::common;
pub use crate::common::algo::*;
use crate::common::gen_array_clone;
pub use crate::common::spdm_codec::*;
use crate::config;

use codec::{Codec, Reader, Writer};

#[derive(Debug, Clone, Default)]
pub struct SpdmNegotiateAlgorithmsRequestPayload {
    pub measurement_specification: SpdmMeasurementSpecification,
    pub base_asym_algo: SpdmBaseAsymAlgo,
    pub base_hash_algo: SpdmBaseHashAlgo,
    pub alg_struct_count: u8,
    pub alg_struct: [SpdmAlgStruct; config::MAX_SPDM_ALG_STRUCT_COUNT],
}

impl SpdmCodec for SpdmNegotiateAlgorithmsRequestPayload {
    fn spdm_encode(&self, _context: &mut common::SpdmContext, bytes: &mut Writer) {
        self.alg_struct_count.encode(bytes); // param1
        0u8.encode(bytes); // param1

        let mut length: u16 = 32;
        for algo in self.alg_struct.iter().take(self.alg_struct_count as usize) {
            length += 2 + algo.alg_fixed_count as u16;
        }
        length.encode(bytes);

        self.measurement_specification.encode(bytes);
        0u8.encode(bytes); // reserved

        self.base_asym_algo.encode(bytes);
        self.base_hash_algo.encode(bytes);
        for _i in 0..12 {
            0u8.encode(bytes); // reserved2
        }

        0u8.encode(bytes); // ext_asym_count

        0u8.encode(bytes); // ext_hash_count

        0u16.encode(bytes); // reserved3

        for algo in self.alg_struct.iter().take(self.alg_struct_count as usize) {
            algo.encode(bytes);
        }
    }

    fn spdm_read(
        _context: &mut common::SpdmContext,
        r: &mut Reader,
    ) -> Option<SpdmNegotiateAlgorithmsRequestPayload> {
        let alg_struct_count = u8::read(r)?; // param1
        u8::read(r)?; // param2

        let length = u16::read(r)?;
        let measurement_specification = SpdmMeasurementSpecification::read(r)?;
        u8::read(r)?; // reserved

        let base_asym_algo = SpdmBaseAsymAlgo::read(r)?;
        let base_hash_algo = SpdmBaseHashAlgo::read(r)?;

        for _i in 0..12 {
            u8::read(r)?; // reserved2
        }

        let ext_asym_count = u8::read(r)?;
        for _ in 0..(ext_asym_count as usize) {
            SpdmExtAlgStruct::read(r)?;
        }

        let ext_hash_count = u8::read(r)?;
        for _ in 0..(ext_hash_count as usize) {
            SpdmExtAlgStruct::read(r)?;
        }

        u16::read(r)?; // reserved3

        let mut alg_struct =
            gen_array_clone(SpdmAlgStruct::default(), config::MAX_SPDM_ALG_STRUCT_COUNT);
        for algo in alg_struct.iter_mut().take(alg_struct_count as usize) {
            *algo = SpdmAlgStruct::read(r)?;
        }

        //
        // check length
        //
        let mut calc_length: u16 = 32 + (4 * ext_asym_count as u16) + (4 * ext_hash_count as u16);
        for alg in alg_struct.iter().take(alg_struct_count as usize) {
            calc_length += 2 + alg.alg_fixed_count as u16 + (4 * alg.alg_ext_count as u16);
        }

        if length != calc_length {
            return None;
        }

        Some(SpdmNegotiateAlgorithmsRequestPayload {
            measurement_specification,
            base_asym_algo,
            base_hash_algo,
            alg_struct_count,
            alg_struct,
        })
    }
}

#[derive(Debug, Clone, Default)]
pub struct SpdmAlgorithmsResponsePayload {
    pub measurement_specification_sel: SpdmMeasurementSpecification,
    pub measurement_hash_algo: SpdmMeasurementHashAlgo,
    pub base_asym_sel: SpdmBaseAsymAlgo,
    pub base_hash_sel: SpdmBaseHashAlgo,
    pub alg_struct_count: u8,
    pub alg_struct: [SpdmAlgStruct; config::MAX_SPDM_ALG_STRUCT_COUNT],
}

impl SpdmCodec for SpdmAlgorithmsResponsePayload {
    fn spdm_encode(&self, _context: &mut common::SpdmContext, bytes: &mut Writer) {
        self.alg_struct_count.encode(bytes); // param1
        0u8.encode(bytes); // param2

        let mut length: u16 = 36;
        for alg in self.alg_struct.iter().take(self.alg_struct_count as usize) {
            length += 2 + alg.alg_fixed_count as u16;
        }
        length.encode(bytes);

        self.measurement_specification_sel.encode(bytes);
        0u8.encode(bytes); // reserved

        self.measurement_hash_algo.encode(bytes);
        self.base_asym_sel.encode(bytes);
        self.base_hash_sel.encode(bytes);
        for _i in 0..12 {
            0u8.encode(bytes); // reserved2
        }

        0u8.encode(bytes); // ext_asym_count

        0u8.encode(bytes); // ext_hash_count

        0u16.encode(bytes); // reserved3

        for algo in self.alg_struct.iter().take(self.alg_struct_count as usize) {
            algo.encode(bytes);
        }
    }

    fn spdm_read(
        _context: &mut common::SpdmContext,
        r: &mut Reader,
    ) -> Option<SpdmAlgorithmsResponsePayload> {
        let alg_struct_count = u8::read(r)?; // param1
        u8::read(r)?; // param2

        let length = u16::read(r)?;

        let measurement_specification_sel = SpdmMeasurementSpecification::read(r)?;
        u8::read(r)?; // reserved

        let measurement_hash_algo = SpdmMeasurementHashAlgo::read(r)?;
        let base_asym_sel = SpdmBaseAsymAlgo::read(r)?;
        let base_hash_sel = SpdmBaseHashAlgo::read(r)?;

        for _i in 0..12 {
            u8::read(r)?; // reserved2
        }

        let ext_asym_count = u8::read(r)?;
        for _ in 0..(ext_asym_count as usize) {
            SpdmExtAlgStruct::read(r)?;
        }

        let ext_hash_count = u8::read(r)?;
        for _ in 0..(ext_hash_count as usize) {
            SpdmExtAlgStruct::read(r)?;
        }

        u16::read(r)?; // reserved3

        let mut alg_struct =
            gen_array_clone(SpdmAlgStruct::default(), config::MAX_SPDM_ALG_STRUCT_COUNT);
        for algo in alg_struct.iter_mut().take(alg_struct_count as usize) {
            *algo = SpdmAlgStruct::read(r)?;
        }

        let mut calc_length: u16 = 36 + (4 * ext_asym_count as u16) + (4 * ext_hash_count as u16);
        for algo in alg_struct.iter().take(alg_struct_count as usize) {
            calc_length += 2 + algo.alg_fixed_count as u16 + (4 * algo.alg_ext_count as u16);
        }

        if length != calc_length {
            return None;
        }

        Some(SpdmAlgorithmsResponsePayload {
            measurement_specification_sel,
            measurement_hash_algo,
            base_asym_sel,
            base_hash_sel,
            alg_struct_count,
            alg_struct,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testlib::*;

    #[test]
    fn test_case0_spdm_negotiate_algorithms_request_payload() {
        let u8_slice = &mut [0u8; 48];
        let mut writer = Writer::init(u8_slice);
        let value = SpdmNegotiateAlgorithmsRequestPayload {
            measurement_specification: SpdmMeasurementSpecification::DMTF,
            base_asym_algo: SpdmBaseAsymAlgo::TPM_ALG_RSASSA_2048,
            base_hash_algo: SpdmBaseHashAlgo::TPM_ALG_SHA_256,
            alg_struct_count: 4,
            alg_struct: gen_array_clone(
                SpdmAlgStruct {
                    alg_type: SpdmAlgType::SpdmAlgTypeDHE,
                    alg_fixed_count: 2,
                    alg_supported: SpdmAlg::SpdmAlgoDhe(SpdmDheAlgo::FFDHE_2048),
                    alg_ext_count: 0,
                },
                config::MAX_SPDM_ALG_STRUCT_COUNT,
            ),
        };
        let pcidoe_transport_encap = &mut PciDoeTransportEncap {};
        let my_spdm_device_io = &mut MySpdmDeviceIo;
        let mut context = new_context(my_spdm_device_io, pcidoe_transport_encap);

        value.spdm_encode(&mut context, &mut writer);
        let mut reader = Reader::init(u8_slice);
        assert_eq!(48, reader.left());
        let spdm_sturct_data =
            SpdmNegotiateAlgorithmsRequestPayload::spdm_read(&mut context, &mut reader).unwrap();
        assert_eq!(
            spdm_sturct_data.measurement_specification,
            SpdmMeasurementSpecification::DMTF
        );
        assert_eq!(
            spdm_sturct_data.base_asym_algo,
            SpdmBaseAsymAlgo::TPM_ALG_RSASSA_2048
        );
        assert_eq!(
            spdm_sturct_data.base_hash_algo,
            SpdmBaseHashAlgo::TPM_ALG_SHA_256
        );
        assert_eq!(spdm_sturct_data.alg_struct_count, 4);
        for i in 0..4 {
            assert_eq!(
                spdm_sturct_data.alg_struct[i].alg_type,
                SpdmAlgType::SpdmAlgTypeDHE
            );
            assert_eq!(spdm_sturct_data.alg_struct[i].alg_fixed_count, 2);
            assert_eq!(
                spdm_sturct_data.alg_struct[1].alg_supported,
                SpdmAlg::SpdmAlgoDhe(SpdmDheAlgo::FFDHE_2048)
            );
            assert_eq!(spdm_sturct_data.alg_struct[i].alg_ext_count, 0);
        }
        assert_eq!(2, reader.left());
    }
    #[test]
    fn test_case1_spdm_negotiate_algorithms_request_payload() {
        let u8_slice = &mut [0u8; 48];
        let mut writer = Writer::init(u8_slice);
        let value = SpdmNegotiateAlgorithmsRequestPayload {
            measurement_specification: SpdmMeasurementSpecification::empty(),
            base_asym_algo: SpdmBaseAsymAlgo::empty(),
            base_hash_algo: SpdmBaseHashAlgo::empty(),
            alg_struct_count: 0,
            alg_struct: gen_array_clone(
                SpdmAlgStruct::default(),
                config::MAX_SPDM_ALG_STRUCT_COUNT,
            ),
        };

        let pcidoe_transport_encap = &mut PciDoeTransportEncap {};
        let my_spdm_device_io = &mut MySpdmDeviceIo;
        let mut context = new_context(my_spdm_device_io, pcidoe_transport_encap);
        value.spdm_encode(&mut context, &mut writer);
        let mut reader = Reader::init(u8_slice);
        assert_eq!(48, reader.left());
        let spdm_sturct_data =
            SpdmNegotiateAlgorithmsRequestPayload::spdm_read(&mut context, &mut reader).unwrap();
        assert_eq!(
            spdm_sturct_data.measurement_specification,
            SpdmMeasurementSpecification::empty()
        );
        assert_eq!(spdm_sturct_data.base_asym_algo, SpdmBaseAsymAlgo::empty());
        assert_eq!(spdm_sturct_data.base_hash_algo, SpdmBaseHashAlgo::empty());
        assert_eq!(spdm_sturct_data.alg_struct_count, 0);
        assert_eq!(18, reader.left());
    }
    #[test]
    fn test_case2_spdm_negotiate_algorithms_request_payload() {
        let u8_slice = &mut [0u8; 48];
        let mut writer = Writer::init(u8_slice);
        let value = SpdmNegotiateAlgorithmsRequestPayload {
            measurement_specification: SpdmMeasurementSpecification::DMTF,
            base_asym_algo: SpdmBaseAsymAlgo::TPM_ALG_RSASSA_2048,
            base_hash_algo: SpdmBaseHashAlgo::TPM_ALG_SHA_256,
            alg_struct_count: 0,
            alg_struct: gen_array_clone(
                SpdmAlgStruct::default(),
                config::MAX_SPDM_ALG_STRUCT_COUNT,
            ),
        };

        let pcidoe_transport_encap = &mut PciDoeTransportEncap {};
        let my_spdm_device_io = &mut MySpdmDeviceIo;
        let mut context = new_context(my_spdm_device_io, pcidoe_transport_encap);
        value.spdm_encode(&mut context, &mut writer);
        u8_slice[26] = 1;
        u8_slice[31] = 1;
        let mut reader = Reader::init(u8_slice);
        assert_eq!(48, reader.left());
        let spdm_negotiate_algorithms_request_payload =
            SpdmNegotiateAlgorithmsRequestPayload::spdm_read(&mut context, &mut reader);
        assert_eq!(spdm_negotiate_algorithms_request_payload.is_none(), true);
        assert_eq!(10, reader.left());
    }
    #[test]
    fn test_case0_spdm_algorithms_response_payload() {
        let u8_slice = &mut [0u8; 50];
        let mut writer = Writer::init(u8_slice);
        let value = SpdmAlgorithmsResponsePayload {
            measurement_specification_sel: SpdmMeasurementSpecification::DMTF,
            measurement_hash_algo: SpdmMeasurementHashAlgo::RAW_BIT_STREAM,
            base_asym_sel: SpdmBaseAsymAlgo::TPM_ALG_RSASSA_2048,
            base_hash_sel: SpdmBaseHashAlgo::TPM_ALG_SHA_256,
            alg_struct_count: 4,
            alg_struct: gen_array_clone(
                SpdmAlgStruct {
                    alg_type: SpdmAlgType::SpdmAlgTypeDHE,
                    alg_fixed_count: 2,
                    alg_supported: SpdmAlg::SpdmAlgoDhe(SpdmDheAlgo::FFDHE_2048),
                    alg_ext_count: 0,
                },
                config::MAX_SPDM_ALG_STRUCT_COUNT,
            ),
        };

        let pcidoe_transport_encap = &mut PciDoeTransportEncap {};
        let my_spdm_device_io = &mut MySpdmDeviceIo;
        let mut context = new_context(my_spdm_device_io, pcidoe_transport_encap);

        value.spdm_encode(&mut context, &mut writer);
        let mut reader = Reader::init(u8_slice);
        assert_eq!(50, reader.left());
        let spdm_sturct_data =
            SpdmAlgorithmsResponsePayload::spdm_read(&mut context, &mut reader).unwrap();
        assert_eq!(
            spdm_sturct_data.measurement_specification_sel,
            SpdmMeasurementSpecification::DMTF
        );
        assert_eq!(
            spdm_sturct_data.measurement_hash_algo,
            SpdmMeasurementHashAlgo::RAW_BIT_STREAM
        );
        assert_eq!(
            spdm_sturct_data.base_asym_sel,
            SpdmBaseAsymAlgo::TPM_ALG_RSASSA_2048
        );
        assert_eq!(
            spdm_sturct_data.base_hash_sel,
            SpdmBaseHashAlgo::TPM_ALG_SHA_256
        );
        assert_eq!(spdm_sturct_data.alg_struct_count, 4);
        for i in 0..4 {
            assert_eq!(
                spdm_sturct_data.alg_struct[i].alg_type,
                SpdmAlgType::SpdmAlgTypeDHE
            );
            assert_eq!(spdm_sturct_data.alg_struct[i].alg_fixed_count, 2);
            assert_eq!(
                spdm_sturct_data.alg_struct[1].alg_supported,
                SpdmAlg::SpdmAlgoDhe(SpdmDheAlgo::FFDHE_2048)
            );
            assert_eq!(spdm_sturct_data.alg_struct[i].alg_ext_count, 0);
        }
        assert_eq!(0, reader.left());
    }
    #[test]
    fn test_case1_spdm_algorithms_response_payload() {
        let u8_slice = &mut [0u8; 48];
        let mut writer = Writer::init(u8_slice);
        let value = SpdmAlgorithmsResponsePayload {
            measurement_specification_sel: SpdmMeasurementSpecification::DMTF,
            measurement_hash_algo: SpdmMeasurementHashAlgo::RAW_BIT_STREAM,
            base_asym_sel: SpdmBaseAsymAlgo::TPM_ALG_RSASSA_2048,
            base_hash_sel: SpdmBaseHashAlgo::TPM_ALG_SHA_256,
            alg_struct_count: 0,
            alg_struct: gen_array_clone(
                SpdmAlgStruct::default(),
                config::MAX_SPDM_ALG_STRUCT_COUNT,
            ),
        };

        let pcidoe_transport_encap = &mut PciDoeTransportEncap {};
        let my_spdm_device_io = &mut MySpdmDeviceIo;
        let mut context = new_context(my_spdm_device_io, pcidoe_transport_encap);
        value.spdm_encode(&mut context, &mut writer);

        u8_slice[30] = 1;
        u8_slice[35] = 1;

        let mut reader = Reader::init(u8_slice);
        assert_eq!(48, reader.left());
        let spdm_algorithms_response_payload =
            SpdmAlgorithmsResponsePayload::spdm_read(&mut context, &mut reader);
        assert_eq!(spdm_algorithms_response_payload.is_none(), true);
    }
    #[test]
    fn test_case2_spdm_algorithms_response_payload() {
        let u8_slice = &mut [0u8; 50];
        let mut writer = Writer::init(u8_slice);
        let value = SpdmAlgorithmsResponsePayload {
            measurement_specification_sel: SpdmMeasurementSpecification::empty(),
            measurement_hash_algo: SpdmMeasurementHashAlgo::empty(),
            base_asym_sel: SpdmBaseAsymAlgo::empty(),
            base_hash_sel: SpdmBaseHashAlgo::empty(),
            alg_struct_count: 0,
            alg_struct: gen_array_clone(
                SpdmAlgStruct::default(),
                config::MAX_SPDM_ALG_STRUCT_COUNT,
            ),
        };

        let pcidoe_transport_encap = &mut PciDoeTransportEncap {};
        let my_spdm_device_io = &mut MySpdmDeviceIo;
        let mut context = new_context(my_spdm_device_io, pcidoe_transport_encap);

        value.spdm_encode(&mut context, &mut writer);
        let mut reader = Reader::init(u8_slice);
        assert_eq!(50, reader.left());
        let spdm_sturct_data =
            SpdmAlgorithmsResponsePayload::spdm_read(&mut context, &mut reader).unwrap();
        assert_eq!(
            spdm_sturct_data.measurement_specification_sel,
            SpdmMeasurementSpecification::empty()
        );
        assert_eq!(
            spdm_sturct_data.measurement_hash_algo,
            SpdmMeasurementHashAlgo::empty()
        );
        assert_eq!(spdm_sturct_data.base_asym_sel, SpdmBaseAsymAlgo::empty());
        assert_eq!(spdm_sturct_data.base_hash_sel, SpdmBaseHashAlgo::empty());
        assert_eq!(spdm_sturct_data.alg_struct_count, 0);
        assert_eq!(16, reader.left());
    }
}
