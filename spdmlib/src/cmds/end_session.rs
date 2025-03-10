// Copyright (c) 2020 Intel Corporation
//
// SPDX-License-Identifier: BSD-2-Clause-Patent

use crate::common;
use crate::msgs::SpdmCodec;
use codec::{Codec, Reader, Writer};

bitflags! {
    #[derive(Default)]
    pub struct SpdmEndSessionRequestAttributes: u8 {
        const PRESERVE_NEGOTIATED_STATE = 0b00000001;
    }
}

impl Codec for SpdmEndSessionRequestAttributes {
    fn encode(&self, bytes: &mut Writer) {
        self.bits().encode(bytes);
    }

    fn read(r: &mut Reader) -> Option<SpdmEndSessionRequestAttributes> {
        let bits = u8::read(r)?;

        SpdmEndSessionRequestAttributes::from_bits(bits)
    }
}

#[derive(Debug, Copy, Clone, Default, PartialEq)]
pub struct SpdmEndSessionRequestPayload {
    pub end_session_request_attributes: SpdmEndSessionRequestAttributes,
}

impl SpdmCodec for SpdmEndSessionRequestPayload {
    fn spdm_encode(&self, _context: &mut common::SpdmContext, bytes: &mut Writer) {
        self.end_session_request_attributes.encode(bytes); // param1
        0u8.encode(bytes); // param2
    }

    fn spdm_read(
        _context: &mut common::SpdmContext,
        r: &mut Reader,
    ) -> Option<SpdmEndSessionRequestPayload> {
        let end_session_request_attributes = SpdmEndSessionRequestAttributes::read(r)?; // param1
        u8::read(r)?; // param2

        Some(SpdmEndSessionRequestPayload {
            end_session_request_attributes,
        })
    }
}

#[derive(Debug, Copy, Clone, Default)]
pub struct SpdmEndSessionResponsePayload {}

impl SpdmCodec for SpdmEndSessionResponsePayload {
    fn spdm_encode(&self, _context: &mut common::SpdmContext, bytes: &mut Writer) {
        0u8.encode(bytes); // param1
        0u8.encode(bytes); // param2
    }

    fn spdm_read(
        _context: &mut common::SpdmContext,
        r: &mut Reader,
    ) -> Option<SpdmEndSessionResponsePayload> {
        u8::read(r)?; // param1
        u8::read(r)?; // param2

        Some(SpdmEndSessionResponsePayload {})
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testlib::*;
    #[test]
    fn test_case0_spdm_response_capability_flags() {
        let u8_slice = &mut [0u8; 1];
        let mut writer = Writer::init(u8_slice);
        let value = SpdmEndSessionRequestAttributes::all();
        value.encode(&mut writer);

        let mut reader = Reader::init(u8_slice);
        assert_eq!(
            SpdmEndSessionRequestAttributes::read(&mut reader).unwrap(),
            SpdmEndSessionRequestAttributes::PRESERVE_NEGOTIATED_STATE
        );
        assert_eq!(0, reader.left());
    }
    #[test]
    fn test_case0_spdm_end_session_request_payload() {
        let u8_slice = &mut [0u8; 12];
        let mut writer = Writer::init(u8_slice);
        let value = SpdmEndSessionRequestPayload {
            end_session_request_attributes:
                SpdmEndSessionRequestAttributes::PRESERVE_NEGOTIATED_STATE,
        };

        let pcidoe_transport_encap = &mut PciDoeTransportEncap {};
        let my_spdm_device_io = &mut MySpdmDeviceIo;
        let mut context = new_context(my_spdm_device_io, pcidoe_transport_encap);

        value.spdm_encode(&mut context, &mut writer);
        let mut reader = Reader::init(u8_slice);
        assert_eq!(12, reader.left());
        let spdm_end_session_request_payload =
            SpdmEndSessionRequestPayload::spdm_read(&mut context, &mut reader).unwrap();
        assert_eq!(
            spdm_end_session_request_payload.end_session_request_attributes,
            SpdmEndSessionRequestAttributes::PRESERVE_NEGOTIATED_STATE
        );
        assert_eq!(10, reader.left());
    }
    #[test]
    fn test_case0_spdm_end_session_response_payload() {
        let u8_slice = &mut [0u8; 8];
        let mut writer = Writer::init(u8_slice);
        let value = SpdmEndSessionResponsePayload {};
        let pcidoe_transport_encap = &mut PciDoeTransportEncap {};
        let my_spdm_device_io = &mut MySpdmDeviceIo;
        let mut context = new_context(my_spdm_device_io, pcidoe_transport_encap);
        value.spdm_encode(&mut context, &mut writer);
        let mut reader = Reader::init(u8_slice);
        SpdmEndSessionResponsePayload::spdm_read(&mut context, &mut reader);
    }
}
